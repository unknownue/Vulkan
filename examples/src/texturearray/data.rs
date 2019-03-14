
use lazy_static::lazy_static;

use ash::vk;

use std::mem;
use std::ptr;
use std::path::{Path, PathBuf};

use vkbase::ci::buffer::BufferCI;
use vkbase::ci::image::{ImageCI, ImageViewCI, ImageBarrierCI, SamplerCI};
use vkbase::ci::pipeline::VertexInputSCI;
use vkbase::ci::vma::{VmaBuffer, VmaImage, VmaAllocationCI};
use vkbase::ci::VkObjectBuildableCI;

use vkbase::context::VkDevice;
use vkbase::command::CmdTransferApi;
use vkbase::FlightCamera;

use vkbase::{vkuint, vkbytes, vkfloat, vkptr, Point3F, Point2F, Vector3F, Vector4F, Matrix4F};
use vkbase::{VkResult, VkError, VkErrorKind};

const TEXTURE_ARRAY_BC3_PATH      : &'static str = "assets/textures/texturearray_bc3_unorm.ktx";
const TEXTURE_ARRAY_ASTC_LDR_PATH : &'static str = "assets/textures/texturearray_astc_8x8_unorm.ktx";
const TEXTURE_ARRAY_ETC2_PATH     : &'static str = "assets/textures/texturearray_etc2_unorm.ktx";

lazy_static! {

    pub static ref VERTEX_DATA: [Vertex; 4] = [
        Vertex { pos: Point3F::new( 2.5,  2.5,  0.0), uv: Point2F::new(1.0, 1.0) }, // v0
        Vertex { pos: Point3F::new(-2.5,  2.5,  0.0), uv: Point2F::new(0.0, 1.0) }, // v1
        Vertex { pos: Point3F::new(-2.5, -2.5,  0.0), uv: Point2F::new(0.0, 0.0) }, // v2
        Vertex { pos: Point3F::new( 2.5, -2.5,  0.0), uv: Point2F::new(1.0, 0.0) }, // v3
    ];

    pub static ref INDEX_DATA: [vkuint; 6] = [0,1,2, 2,3,0];
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pos: Point3F,
    uv : Point2F,
}

impl Vertex {

    pub fn input_description() -> VertexInputSCI {

        VertexInputSCI::new()
            .add_binding(vk::VertexInputBindingDescription {
                binding: 0,
                stride : ::std::mem::size_of::<Vertex>() as _,
                input_rate: vk::VertexInputRate::VERTEX,
            })
            // location 0: Position.
            .add_attribute(vk::VertexInputAttributeDescription {
                location: 0,
                binding : 0,
                format  : vk::Format::R32G32B32_SFLOAT,
                offset  : memoffset::offset_of!(Vertex, pos) as _,
            })
            // location 1: Texture coordinates.
            .add_attribute(vk::VertexInputAttributeDescription {
                location: 1,
                binding : 0,
                format  : vk::Format::R32G32_SFLOAT,
                offset  : memoffset::offset_of!(Vertex, uv) as _,
            })
    }
}

pub fn generate_quad(device: &mut VkDevice) -> VkResult<(VmaBuffer, VmaBuffer)> {

    // Setup vertices for a single uv-mapped quad made from two triangles.
    // For the sake of simplicity, we won't stage the vertex data to the gpu memory.

    let vertex_buffer = {

        let vertices_ci = BufferCI::new((mem::size_of::<Vertex>() * VERTEX_DATA.len()) as vkbytes)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER);
        let allocation_ci = VmaAllocationCI::new(vma::MemoryUsage::CpuOnly, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)
            .flags(vma::AllocationCreateFlags::MAPPED);
        let vertices_allocation = device.vma.create_buffer(
            &vertices_ci.value(), allocation_ci.as_ref())
            .map_err(VkErrorKind::Vma)?;

        unsafe {
            let data_ptr = vertices_allocation.2.get_mapped_data() as vkptr<Vertex>;
            debug_assert_ne!(data_ptr, ptr::null_mut());
            data_ptr.copy_from_nonoverlapping(VERTEX_DATA.as_ptr(), VERTEX_DATA.len());
        }

        VmaBuffer::from(vertices_allocation)
    };

    let index_buffer = {

        let indices_ci = BufferCI::new((mem::size_of::<vkuint>() * INDEX_DATA.len()) as vkbytes)
            .usage(vk::BufferUsageFlags::INDEX_BUFFER);
        let allocation_ci = VmaAllocationCI::new(vma::MemoryUsage::CpuOnly, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)
            .flags(vma::AllocationCreateFlags::MAPPED);
        let indices_allocation = device.vma.create_buffer(
            &indices_ci.value(), allocation_ci.as_ref())
            .map_err(VkErrorKind::Vma)?;

        unsafe {

            let data_ptr = indices_allocation.2.get_mapped_data() as vkptr<vkuint>;
            debug_assert_ne!(data_ptr, ptr::null_mut());
            data_ptr.copy_from_nonoverlapping(INDEX_DATA.as_ptr(), INDEX_DATA.len());
        }

        VmaBuffer::from(indices_allocation)
    };

    Ok((vertex_buffer, index_buffer))
}


#[derive(Debug)]
pub struct UboVS {
    pub matrices: UboMatrices,
    // Separate data for each instance.
    pub instances: Vec<UboInstanceData>,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct UboMatrices {
    pub projection: Matrix4F,
    pub view      : Matrix4F,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct UboInstanceData {
    // model matrix.
    pub model: Matrix4F,
    // Texture array index(Vec4 due to padding).
    pub array_index: Vector4F,
}

impl UboVS {

    pub fn prepare_buffer(device: &mut VkDevice, camera: &FlightCamera, textures: &TextureArray) -> VkResult<(VmaBuffer, UboVS)> {

        let ubo_size = mem::size_of::<UboMatrices>() + (textures.layer_count as usize * mem::size_of::<UboInstanceData>());

        let ubo_buffer = {

            let ubo_ci = BufferCI::new(ubo_size as vkbytes)
                .usage(vk::BufferUsageFlags::UNIFORM_BUFFER);
            let allocation_ci = VmaAllocationCI::new(vma::MemoryUsage::CpuOnly, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)
                .flags(vma::AllocationCreateFlags::MAPPED);
            let ubo_allocation = device.vma.create_buffer(
                &ubo_ci.value(), allocation_ci.as_ref())
                .map_err(VkErrorKind::Vma)?;

            VmaBuffer::from(ubo_allocation)
        };

        let ubo_data = {

            let mut ubo_data = UboVS {
                matrices: UboMatrices {
                    projection: camera.proj_matrix(),
                    view      : camera.view_matrix(),
                },
                instances: Vec::new(),
            };

            // Array indices and model matrices are fixed.
            const OFFSET: vkfloat = -5.0;
            let center = (textures.layer_count as vkfloat * OFFSET) / 2.0;

            // Update instanced part of the uniform buffer.
            for i in 0..textures.layer_count {
                // instance model matrix.
                let instance_data = UboInstanceData {
                    model: Matrix4F::new_translation(&Vector3F::new(0.0, (i as f32) * OFFSET - center, 0.0)),
                    // * Matrix4F::from_axis_angle(&Vector3F::x_axis(), ::std::f32::consts::FRAC_PI_3)
                    array_index: Vector4F::new(i as f32, 0.0, 0.0, 0.0),
                };
                ubo_data.instances.push(instance_data);
            }

            ubo_data
        };

        unsafe {
            let data_ptr = ubo_buffer.info.get_mapped_data() as vkptr<UboMatrices>;
            debug_assert_ne!(data_ptr, ptr::null_mut());
            data_ptr.copy_from_nonoverlapping(&ubo_data.matrices, 1);

            let instance_ptr = data_ptr.offset(1) as vkptr<UboInstanceData>;
            instance_ptr.copy_from_nonoverlapping(ubo_data.instances.as_ptr(), ubo_data.instances.len());
        }

        Ok((ubo_buffer, ubo_data))
    }
}

/// `Texture` contains all Vulkan objects that are required to store and use a texture.
pub struct TextureArray {
    pub sampler: vk::Sampler,
    pub image  : VmaImage,
    pub view   : vk::ImageView,
    pub layout : vk::ImageLayout,

    pub width : vkuint,
    pub height: vkuint,
    pub layer_count: vkuint,
}


impl TextureArray {

    pub fn load(device: &mut VkDevice) -> VkResult<TextureArray> {

        // Sascha Willems's comment:
        // Vulkan core supports three different compressed texture formats.
        // As the support differs between implementations, we need to check device features and select a proper format and file.

        let (texture_path, texture_format) = if device.phy.features_enabled().texture_compression_bc == vk::TRUE {
            (PathBuf::from(TEXTURE_ARRAY_BC3_PATH), vk::Format::BC3_UNORM_BLOCK)
        } else if device.phy.features_enabled().texture_compression_astc_ldr == vk::TRUE {
            (PathBuf::from(TEXTURE_ARRAY_ASTC_LDR_PATH), vk::Format::ASTC_8X8_UNORM_BLOCK)
        } else if device.phy.features_enabled().texture_compression_etc2 == vk::TRUE {
            (PathBuf::from(TEXTURE_ARRAY_ETC2_PATH), vk::Format::ETC2_R8G8B8_UNORM_BLOCK)
        } else {
            return Err(VkError::unsupported("Compressed texture format"))
        };

        TextureArray::load_ktx(device, texture_path, texture_format)
    }

    fn load_ktx(device: &mut VkDevice, texture_path: impl AsRef<Path>, format: vk::Format) -> VkResult<TextureArray> {

        use gli::GliTexture;

        let tex_2d_array: gli::Texture2DArray = gli::load_ktx(texture_path)
            .map_err(VkErrorKind::Gli)?;

        debug_assert!(!tex_2d_array.empty());

        let width  = tex_2d_array.extent(0).width;
        let height = tex_2d_array.extent(0).height;
        let layer_count = tex_2d_array.layers() as vkuint;


        let staging_buffer = {

            // create a host-visible staging buffer that contains the raw image data.
            // this buffer will be the data source for copying texture data to the optimal tiled image on the device.

            let staging_ci = BufferCI::new(tex_2d_array.size() as vkbytes)
                .usage(vk::BufferUsageFlags::TRANSFER_SRC);
            let allocation_ci = VmaAllocationCI::new(vma::MemoryUsage::CpuOnly, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
            let staging_allocation = device.vma.create_buffer(
                &staging_ci.value(), allocation_ci.as_ref())
                .map_err(VkErrorKind::Vma)?;

            // Copy texture data into host local staging buffer.
            let data_ptr = device.vma.map_memory(&staging_allocation.1)
                .map_err(VkErrorKind::Vma)?;
            debug_assert_ne!(data_ptr, ptr::null_mut());

            unsafe {
                data_ptr.copy_from(tex_2d_array.data() as *const u8, tex_2d_array.size());
            }

            device.vma.unmap_memory(&staging_allocation.1)
                .map_err(VkErrorKind::Vma)?;

            VmaBuffer::from(staging_allocation)
        };

        // setup buffer copy regions for each mip level.
        let mut buffer_copy_regions = Vec::with_capacity(layer_count as usize);
        let mut staging_offset = 0;

        for i in 0..layer_count {

            // Get a layer(Texture2D) from Texture2DArray.
            let texture_layer_i: gli::Texture2D = tex_2d_array.get_layer(i as usize);
            // Get the base mip-level image of this layer.
            let base_level_image: gli::GliImage = texture_layer_i.get_level(0);

            let copy_region = vk::BufferImageCopy {
                buffer_offset: staging_offset,
                // specify the following two member to 0 to tell vulkan the image is tightly packed.
                buffer_row_length  : 0,
                buffer_image_height: 0,
                image_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: 0,
                    base_array_layer: i,
                    layer_count     : 1,
                },
                image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
                image_extent: vk::Extent3D {
                    width : base_level_image.extent().width,
                    height: base_level_image.extent().height,
                    depth : 1,
                },
            };

            buffer_copy_regions.push(copy_region);
            staging_offset += base_level_image.size() as vkbytes;
        }

        // create optimal tiled target image on the device.
        let dst_image = {

            let image_ci = ImageCI::new_2d(format, vk::Extent2D { width, height })
                .mip_levels(1)
                .array_layers(layer_count)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .usages(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED);

            let allocation_ci = VmaAllocationCI::new(
                vma::MemoryUsage::GpuOnly, vk::MemoryPropertyFlags::DEVICE_LOCAL);
            let image_allocation = device.vma.create_image(
                &image_ci.value(), allocation_ci.as_ref())
                .map_err(VkErrorKind::Vma)?;

            VmaImage::from(image_allocation)
        };


        {
            let sub_range = vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count   : 1,
                base_array_layer: 0,
                layer_count,
            };

            // image memory barriers for the texture. -------------------
            // set initial layout for all array layers of the optimal (target) titled texture.

            let barrier1 = ImageBarrierCI::new(dst_image.handle, sub_range)
                .access_mask(vk::AccessFlags::empty(), vk::AccessFlags::TRANSFER_WRITE)
                .layout(vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL);

            let barrier2 = ImageBarrierCI::new(dst_image.handle, sub_range)
                .access_mask(vk::AccessFlags::TRANSFER_WRITE, vk::AccessFlags::SHADER_READ)
                .layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
            // ----------------------------------------------------------

            // transfer data from staging buffer to dst image.
            let copy_recorder = device.get_transfer_recorder();

            copy_recorder.begin_record()?
                .image_pipeline_barrier(vk::PipelineStageFlags::HOST, vk::PipelineStageFlags::TRANSFER, vk::DependencyFlags::empty(), &[barrier1.value()])
                // Copy all layers from staging buffer.
                .copy_buf2img(staging_buffer.handle, dst_image.handle, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &buffer_copy_regions)
                .image_pipeline_barrier(vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::ALL_COMMANDS, vk::DependencyFlags::empty(), &[barrier2.value()])
                .end_record()?;

            device.flush_transfer(copy_recorder)?;
        }


        { // clean up staging resources.
            device.vma_discard(staging_buffer)?;
        }

        let dst_sampler = {

            let mut sampler_ci = SamplerCI::new()
                .filter(vk::Filter::LINEAR, vk::Filter::LINEAR)
                .mipmap(vk::SamplerMipmapMode::LINEAR)
                .address(vk::SamplerAddressMode::CLAMP_TO_EDGE, vk::SamplerAddressMode::CLAMP_TO_EDGE, vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .lod(0.0, 0.0, 0.0)
                .compare_op(Some(vk::CompareOp::NEVER))
                .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE);

            if device.phy.features_enabled().sampler_anisotropy == vk::TRUE {
                sampler_ci = sampler_ci.anisotropy(Some(8.0));
            } else {
                sampler_ci = sampler_ci.anisotropy(None);
            }

            sampler_ci.build(device)?
        };

        let dst_image_view = {

            ImageViewCI::new(dst_image.handle, vk::ImageViewType::TYPE_2D_ARRAY, format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::R,
                    g: vk::ComponentSwizzle::G,
                    b: vk::ComponentSwizzle::B,
                    a: vk::ComponentSwizzle::A,
                })
                .sub_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count   : 1,
                    base_array_layer: 0,
                    layer_count,
                }).build(device)?
        };


        let result = TextureArray {
            sampler: dst_sampler,
            image  : dst_image,
            view   : dst_image_view,
            layout : vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            width, height, layer_count,
        };
        Ok(result)
    }

    pub fn discard_by(self, device: &mut VkDevice) -> VkResult<()> {

        device.discard(self.sampler);
        device.discard(self.view);
        device.vma_discard(self.image)
    }
}

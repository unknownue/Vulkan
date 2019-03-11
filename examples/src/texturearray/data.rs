
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

use vkbase::{vkuint, vkbytes, vkfloat, vkptr, Point4F, Point3F, Point2F, Vector3F, Matrix4F};
use vkbase::{VkResult, VkError, VkErrorKind};

const TEXTURE_ARRAY_BC3_PATH      : &'static str = "assets/textures/texturearray_bc3_unorm.ktx";
const TEXTURE_ARRAY_ASTC_LDR_PATH : &'static str = "assets/textures/texturearray_astc_8x8_unorm.ktx";
const TEXTURE_ARRAY_ETC2_PATH     : &'static str = "assets/textures/texturearray_etc2_unorm.ktx";

lazy_static! {

    pub static ref VERTEX_DATA: [Vertex; 4] = [
        Vertex { pos: Point3F::new( 1.0,  1.0,  0.0), uv: Point2F::new(1.0, 1.0), normal: Vector3F::new(0.0, 0.0, 1.0) }, // v0
        Vertex { pos: Point3F::new(-1.0,  1.0,  0.0), uv: Point2F::new(0.0, 1.0), normal: Vector3F::new(0.0, 0.0, 1.0) }, // v1
        Vertex { pos: Point3F::new(-1.0, -1.0,  0.0), uv: Point2F::new(0.0, 0.0), normal: Vector3F::new(0.0, 0.0, 1.0) }, // v2
        Vertex { pos: Point3F::new( 1.0, -1.0,  0.0), uv: Point2F::new(1.0, 0.0), normal: Vector3F::new(0.0, 0.0, 1.0) }, // v3
    ];

    pub static ref INDEX_DATA: [vkuint; 6] = [0,1,2, 2,3,0];
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pos: Point3F,
    uv : Point2F,
    normal: Vector3F,
}

impl Vertex {

    pub fn input_description() -> VertexInputSCI {

        VertexInputSCI::new()
            .add_binding(vk::VertexInputBindingDescription {
                binding: 0,
                stride : ::std::mem::size_of::<Vertex>() as _,
                input_rate: vk::VertexInputRate::VERTEX,
            })
            .add_attribute(vk::VertexInputAttributeDescription {
                location: 0,
                binding : 0,
                format  : vk::Format::R32G32B32_SFLOAT,
                offset  : memoffset::offset_of!(Vertex, pos) as _,
            })
            .add_attribute(vk::VertexInputAttributeDescription {
                location: 1,
                binding : 0,
                format  : vk::Format::R32G32_SFLOAT,
                offset  : memoffset::offset_of!(Vertex, uv) as _,
            }).add_attribute(vk::VertexInputAttributeDescription {
                location: 2,
                binding : 0,
                format  : vk::Format::R32G32B32_SFLOAT,
                offset  : memoffset::offset_of!(Vertex, normal) as _,
            })
    }
}

pub fn generate_quad(device: &mut VkDevice) -> VkResult<(VmaBuffer, VmaBuffer)> {

    // setup vertices for a single uv-mapped quad made from two triangles.
    // for the sake of simplicity, we won't stage the vertex data to the gpu memory.

    use vkbase::utils::memory::copy_to_ptr;

    let vertex_buffer = {

        let vertices_ci = BufferCI::new((mem::size_of::<Vertex>() * VERTEX_DATA.len()) as vkbytes)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER);
        let allocation_ci = VmaAllocationCI::new(vma::MemoryUsage::CpuOnly, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)
            .flags(vma::AllocationCreateFlags::MAPPED);
        let vertices_allocation = device.vma.create_buffer(
            &vertices_ci.value(), allocation_ci.as_ref())
            .map_err(VkErrorKind::Vma)?;

        let data_ptr = vertices_allocation.2.get_mapped_data() as vkptr;
        debug_assert_ne!(data_ptr, ptr::null_mut());
        copy_to_ptr(data_ptr, VERTEX_DATA.as_ref());

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

        let data_ptr = indices_allocation.2.get_mapped_data() as vkptr;
        debug_assert_ne!(data_ptr, ptr::null_mut());
        copy_to_ptr(data_ptr, INDEX_DATA.as_ref());

        VmaBuffer::from(indices_allocation)
    };

    Ok((vertex_buffer, index_buffer))
}


#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct UboVS {
    pub projection: Matrix4F,
    pub model     : Matrix4F,
    pub view_pos  : Point4F,
    pub lod_bias  : f32,
}

pub struct UboVSData {
    pub content: [UboVS; 1],
}

impl UboVSData {

    pub fn prepare_buffer(device: &mut VkDevice, camera: &FlightCamera) -> VkResult<(VmaBuffer, UboVSData)> {

        let buffer_ci = BufferCI::new(mem::size_of::<UboVSData>() as vkbytes)
            .usage(vk::BufferUsageFlags::UNIFORM_BUFFER);
        let allocation_ci = VmaAllocationCI::new(vma::MemoryUsage::CpuOnly, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)
            .flags(vma::AllocationCreateFlags::MAPPED);
        let buffer_allocation = device.vma.create_buffer(
            &buffer_ci.value(), allocation_ci.as_ref())
            .map_err(VkErrorKind::Vma)?;

        let ubo_data = UboVSData {
            content: [
                UboVS {
                    projection: camera.proj_matrix(),
                    model     : Matrix4F::identity(),
                    view_pos  : Point4F::new(0.0, 0.0, -2.5, 0.0),
                    lod_bias  : 0.0,
                },
            ],
        };

        let data_ptr = buffer_allocation.2.get_mapped_data() as vkptr;
        debug_assert_ne!(data_ptr, ptr::null_mut());
        vkbase::utils::memory::copy_to_ptr(data_ptr, &ubo_data.content);

        Ok((VmaBuffer::from(buffer_allocation), ubo_data))
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

    fn load(device: &mut VkDevice) -> VkResult<TextureArray> {

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

        let width  = tex_2d_array.extent(0)[0];
        let height = tex_2d_array.extent(0)[1];
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
        let mut buffer_copy_regions = Vec::with_capacity(tex_2d.levels());
        let mut staging_offset = 0;

        for i in 0..layer_count {

            // Get a layer(Texture2D) from Texture2DArray.
            let texture_layer_i: gli::Texture2D = tex_2d_array.get_layer(i);
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
                    width : base_level_image.extent()[0],
                    height: base_level_image.extent()[1],
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
                .image_pipeline_barrier(vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::FRAGMENT_SHADER, vk::DependencyFlags::empty(), &[barrier2.value()])
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
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .mip_level(0, 1)
                .array_layers(0, layer_count)
                .build(device)?
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

    #[inline]
    pub fn descriptor(&self) -> vk::DescriptorImageInfo {
        vk::DescriptorImageInfo {
            sampler      : self.sampler,
            image_view   : self.view,
            image_layout : self.layout,
        }
    }

    pub fn discard_by(self, device: &mut VkDevice) -> VkResult<()> {

        device.discard(self.sampler);
        device.discard(self.view);
        device.vma_discard(self.image)
    }
}

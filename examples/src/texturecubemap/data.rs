
use ash::vk;

use std::mem;
use std::ptr;
use std::path::{Path, PathBuf};

use vkbase::ci::buffer::BufferCI;
use vkbase::ci::image::{ImageCI, ImageViewCI, ImageBarrierCI, SamplerCI};
use vkbase::ci::vma::{VmaBuffer, VmaImage, VmaAllocationCI};
use vkbase::ci::VkObjectBuildableCI;

use vkbase::context::VkDevice;
use vkbase::gltf::VkglTFModel;
use vkbase::command::CmdTransferApi;
use vkbase::FlightCamera;

use vkbase::{vkuint, vkbytes, vkfloat, Matrix4F};
use vkbase::{VkResult, VkError, VkErrorKind};

const CUBEMAP_TEXTURE_COMPRESSION_BC_PATH       : &'static str = "assets/textures/cubemap_yokohama_bc3_unorm.ktx";
const CUBEMAP_TEXTURE_COMPRESSION_ASTC_LDR_PATH : &'static str = "assets/textures/cubemap_yokohama_astc_8x8_unorm.ktx";
const CUBEMAP_TEXTURE_COMPRESSION_ETC2_PATH     : &'static str = "assets/textures/cubemap_yokohama_etc2_unorm.ktx";
// There are 6 faces for each cube.
const CUBE_FACES_COUNT: usize = 6;
const CUBE_MODEL_PATH: &'static str = "assets/models/cube.gltf";


pub struct Skybox {

    pub model: VkglTFModel,
    pub texture: TextureCube,

    pub ubo_buffer: VmaBuffer,
    pub ubo_data: UBOVS,

    pub descriptor_set: vk::DescriptorSet,
}

impl Skybox {

    pub fn load_meshes(device: &mut VkDevice, camera: &FlightCamera) -> VkResult<Skybox> {

        use vkbase::gltf::{GltfModelInfo, load_gltf};
        use vkbase::gltf::{AttributeFlags, NodeAttachmentFlags};

        let model_info = GltfModelInfo {
            path: Path::new(CUBE_MODEL_PATH),
            // specify model's vertices layout.
            // in skybox.vert.glsl:
            //
            // layout (location = 0) in vec3 inPos;
            attribute: AttributeFlags::POSITION,
            // specify model's node attachment layout.
            // in skybox.vert.glsl:
            //
            // layout (set = 0, binding = 1) uniform DynNode {
            //     mat4 transform;
            // } dyn_node;
            node: NodeAttachmentFlags::TRANSFORM_MATRIX,
            transform: None,
        };

        let (ubo_buffer, ubo_data) = UBOVS::prepare_buffer(device, camera)?;

        let skybox_meshes = Skybox {
            model: load_gltf(device, model_info)?,
            texture: load_skybox_textures(device)?,
            descriptor_set: vk::DescriptorSet::null(),
            ubo_buffer, ubo_data,
        };
        Ok(skybox_meshes)
    }

    pub fn discard_by(self, device: &mut VkDevice) -> VkResult<()> {

        device.vma_discard(self.ubo_buffer)?;

        self.texture.discard_by(device)?;
        device.vma_discard(self.model)
    }
}





/*
    layout (binding = 0) uniform UBO  {
        mat4 projection;
        mat4 model;
        float lodBias;
    } ubo;
*/
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct UBOVS {
    pub projection: Matrix4F,
    pub model     : Matrix4F,
    pub lod_bias  : f32,
}

impl UBOVS {

    fn prepare_buffer(device: &mut VkDevice, camera: &FlightCamera) -> VkResult<(VmaBuffer, UBOVS)> {

        let buffer_ci = BufferCI::new(mem::size_of::<UBOVS>() as vkbytes)
            .usage(vk::BufferUsageFlags::UNIFORM_BUFFER);
        let allocation_ci = VmaAllocationCI::new(vma::MemoryUsage::CpuOnly, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)
            .flags(vma::AllocationCreateFlags::MAPPED);
        let buffer_allocation = device.vma.create_buffer(
            &buffer_ci, &allocation_ci)
            .map_err(VkErrorKind::Vma)?;

        let ubo_data = UBOVS {
            projection: camera.proj_matrix(),
            model     : camera.view_matrix(),
            lod_bias  : 0.0,
        };

        Ok((VmaBuffer::from(buffer_allocation), ubo_data))
    }
}

/// `TextureCube` contains all Vulkan objects that are required to store and use a texture cube.
pub struct TextureCube {
    pub sampler: vk::Sampler,
    pub image  : VmaImage,
    pub view   : vk::ImageView,
    pub layout : vk::ImageLayout,

    pub width : vkuint,
    pub height: vkuint,
    pub mip_levels: vkuint,
}

fn load_skybox_textures(device: &mut VkDevice) -> VkResult<TextureCube> {

    // Sascha Willems's comment:
    // Vulkan core supports three different compressed texture formats.
    // As the support differs between implementations, we need to check device features and select a proper format and file.

    let (texture_path, texture_format) = if device.phy.features_enabled().texture_compression_bc == vk::TRUE {
        (PathBuf::from(CUBEMAP_TEXTURE_COMPRESSION_BC_PATH), vk::Format::BC2_UNORM_BLOCK)
    } else if device.phy.features_enabled().texture_compression_astc_ldr == vk::TRUE {
        (PathBuf::from(CUBEMAP_TEXTURE_COMPRESSION_ASTC_LDR_PATH), vk::Format::ASTC_8X8_UNORM_BLOCK)
    } else if device.phy.features_enabled().texture_compression_etc2 == vk::TRUE {
        (PathBuf::from(CUBEMAP_TEXTURE_COMPRESSION_ETC2_PATH), vk::Format::ETC2_R8G8B8_UNORM_BLOCK)
    } else {
        return Err(VkError::unsupported("Compressed texture format"))
    };

    TextureCube::load_ktx(device, texture_path, texture_format)
}

impl TextureCube {

    pub fn load_ktx(device: &mut VkDevice, texture_path: impl AsRef<Path>, format: vk::Format) -> VkResult<TextureCube> {

        use gli::GliTexture;

        let tex_cube: gli::TextureCube = gli::load_ktx(texture_path)
            .map_err(VkErrorKind::Gli)?;

        debug_assert!(!tex_cube.empty());

        let (width, height, mip_levels) = {
            // get the base level image in this texture.
            let base_extent = tex_cube.extent(0);
            (base_extent.width, base_extent.height, tex_cube.levels() as vkuint)
        };

        let staging_buffer = {

            // create a host-visible staging buffer that contains the raw image data.
            // this buffer will be the data source for copying texture data to the optimal tiled image on the device.

            let staging_ci = BufferCI::new(tex_cube.size() as vkbytes)
                .usage(vk::BufferUsageFlags::TRANSFER_SRC);
            let allocation_ci = VmaAllocationCI::new(vma::MemoryUsage::CpuOnly, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
            let staging_allocation = device.vma.create_buffer(
                &staging_ci, &allocation_ci)
                .map_err(VkErrorKind::Vma)?;

            // Copy texture data into host local staging buffer.
            let data_ptr = device.vma.map_memory(&staging_allocation.1)
                .map_err(VkErrorKind::Vma)?;
            debug_assert_ne!(data_ptr, ptr::null_mut());

            unsafe {
                data_ptr.copy_from(tex_cube.data() as *const u8, tex_cube.size());
            }

            device.vma.unmap_memory(&staging_allocation.1)
                .map_err(VkErrorKind::Vma)?;

            VmaBuffer::from(staging_allocation)
        };

        // create optimal tiled target image on the device.
        let dst_image = {

            let image_ci = ImageCI::new_2d(format, vk::Extent2D { width, height })
                // This flag is required for cube map images.
                .flags(vk::ImageCreateFlags::CUBE_COMPATIBLE)
                // Cube faces count as array layers in Vulkan.
                .array_layers(CUBE_FACES_COUNT as vkuint)
                .mip_levels(mip_levels)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .usages(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED);

            let allocation_ci = VmaAllocationCI::new(
                vma::MemoryUsage::GpuOnly, vk::MemoryPropertyFlags::DEVICE_LOCAL);
            let image_allocation = device.vma.create_image(
                &image_ci, &allocation_ci)
                .map_err(VkErrorKind::Vma)?;

            VmaImage::from(image_allocation)
        };


        // setup buffer copy regions for each face including all of it's mip level.
        let mut buffer_copy_regions = Vec::with_capacity(tex_cube.levels() * CUBE_FACES_COUNT);
        let mut staging_offset = 0;

        for face in 0..CUBE_FACES_COUNT {

            let cube_face = tex_cube.get_face(face);

            for i in 0..tex_cube.levels() {

                let face_level_i = cube_face.get_level(i);

                let copy_region = vk::BufferImageCopy {
                    buffer_offset: staging_offset,
                    // specify the following two member to 0 to tell vulkan the image is tightly packed.
                    buffer_row_length  : 0,
                    buffer_image_height: 0,
                    image_subresource: vk::ImageSubresourceLayers {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        mip_level: i as vkuint,
                        base_array_layer: face as vkuint,
                        layer_count     : 1,
                    },
                    image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
                    image_extent: vk::Extent3D {
                        width : face_level_i.extent().width,
                        height: face_level_i.extent().height,
                        depth : 1,
                    },
                };

                buffer_copy_regions.push(copy_region);
                // Increase offset into staging buffer for next level/face.
                staging_offset += face_level_i.size() as vkbytes;
            }
        }


        {
            // Set barrier range between levels and layers across all the cube map image.
            let sub_range = vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count   : mip_levels,
                base_array_layer: 0,
                layer_count     : CUBE_FACES_COUNT as vkuint,
            };

            // image memory barriers for the texture image. -------------

            let barrier1 = ImageBarrierCI::new(dst_image.handle, sub_range)
                .access_mask(vk::AccessFlags::empty(), vk::AccessFlags::TRANSFER_WRITE)
                .layout(vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL);

            // change texture image layout to shader read after all faces have been copied.
            let barrier2 = ImageBarrierCI::new(dst_image.handle, sub_range)
                .access_mask(vk::AccessFlags::TRANSFER_WRITE, vk::AccessFlags::SHADER_READ)
                .layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
            // ----------------------------------------------------------

            // transfer data from staging buffer to dst image.
            let copy_recorder = device.get_transfer_recorder();

            copy_recorder.begin_record()?
                .image_pipeline_barrier(vk::PipelineStageFlags::HOST, vk::PipelineStageFlags::TRANSFER, vk::DependencyFlags::empty(), &[barrier1.value()])
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
                .lod(0.0, 0.0, mip_levels as vkfloat)
                .compare_op(Some(vk::CompareOp::NEVER))
                .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE);

            if device.phy.features_enabled().sampler_anisotropy == vk::TRUE {
                sampler_ci = sampler_ci.anisotropy(Some(device.phy.limits.max_sampler_anisotropy));
            } else {
                sampler_ci = sampler_ci.anisotropy(None);
            }

            sampler_ci.build(device)?
        };

        let dst_image_view = {

            ImageViewCI::new(dst_image.handle, vk::ImageViewType::CUBE, format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::R,
                    g: vk::ComponentSwizzle::G,
                    b: vk::ComponentSwizzle::B,
                    a: vk::ComponentSwizzle::A,
                })
                .sub_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    // set number of mip levels.
                    level_count: mip_levels,
                    base_array_layer: 0,
                    // 6 array layers(faces)
                    layer_count: CUBE_FACES_COUNT as vkuint,
                }).build(device)?
        };


        let result = TextureCube {
            sampler: dst_sampler,
            image  : dst_image,
            view   : dst_image_view,
            layout : vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            width, height, mip_levels,
        };
        Ok(result)
    }

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


use lazy_static::lazy_static;

use ash::vk;

use std::mem;
use std::ptr;
use std::path::Path;

use vkbase::ci::buffer::BufferCI;
use vkbase::ci::image::{ImageCI, ImageViewCI, ImageBarrierCI, SamplerCI};
use vkbase::ci::pipeline::VertexInputSCI;
use vkbase::ci::vma::{VmaBuffer, VmaImage, VmaAllocationCI};
use vkbase::ci::command::{CommandPoolCI, CommandBufferAI};
use vkbase::ci::VkObjectBuildableCI;

use vkbase::context::VkDevice;
use vkbase::command::{VkCmdRecorder, ITransfer, CmdTransferApi};
use vkbase::FlightCamera;

use vkbase::{vkuint, vkbytes, vkfloat, vkptr, Point4F, Point3F, Point2F, Vector3F, Matrix4F};
use vkbase::{VkResult, VkErrorKind};


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
        device.copy_to_ptr(data_ptr, VERTEX_DATA.as_ref());

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
        device.copy_to_ptr(data_ptr, INDEX_DATA.as_ref());

        VmaBuffer::from(indices_allocation)
    };

    Ok((vertex_buffer, index_buffer))
}


#[derive(Debug, Clone, Copy)]
pub struct UboVS {
    pub projection: Matrix4F,
    pub view      : Matrix4F,
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
                    view      : camera.view_matrix(),
                    model     : Matrix4F::identity(),
                    view_pos  : Point4F::new(0.0, 0.0, -2.5, 0.0),
                    lod_bias  : 0.0,
                },
            ],
        };

        let data_ptr = buffer_allocation.2.get_mapped_data() as vkptr;
        debug_assert_ne!(data_ptr, ptr::null_mut());
        device.copy_to_ptr(data_ptr, &ubo_data.content);

        Ok((VmaBuffer::from(buffer_allocation), ubo_data))
    }
}

/// `Texture` contains all Vulkan objects that are required to store and use a texture.
pub struct Texture {
    pub sampler: vk::Sampler,
    pub image  : VmaImage,
    pub view   : vk::ImageView,
    pub layout : vk::ImageLayout,

    pub width : vkuint,
    pub height: vkuint,
    pub mip_levels: vkuint,
}

// The following description is copied from SaschaWillems's repository.
/*
    Upload texture image data to the GPU

    Vulkan offers two types of image tiling (memory layout):

    Linear tiled images:
        These are stored as is and can be copied directly to. But due to the linear nature they're not a good match for GPUs and format and feature support is very limited.
        It's not advised to use linear tiled images for anything else than copying from host to GPU if buffer copies are not an option.
        Linear tiling is thus only implemented for learning purposes, one should always prefer optimal tiled image.

    Optimal tiled images:
        These are stored in an implementation specific layout matching the capability of the hardware. They usually support more formats and features and are much faster.
        Optimal tiled images are stored on the device and not accessible by the host. So they can't be written directly to (like liner tiled images) and always require
        some sort of data copy, either from a buffer or	a linear tiled image.

    In Short: Always use optimal tiled images for rendering.
*/

impl Texture {

    pub fn load(device: &mut VkDevice, texture_path: impl AsRef<Path>) -> VkResult<Texture> {

        use gli::GliTexture;

        // For more detail about ktx format, visit https://www.khronos.org/opengles/sdk/tools/KTX/file_format_spec/ .
        // Texture data contains 4 channels (RGBA) with unnormalized 8-bit values, this is the most commonly supported format.
        let format = vk::Format::R8G8B8A8_UNORM;

        let tex_2d: gli::Texture2D = gli::load_ktx(texture_path)
            .map_err(VkErrorKind::Gli)?;

        debug_assert!(!tex_2d.empty());

        let (width, height) = {
            // get the base level image in this texture.
            let base_image = tex_2d.get_level(0);
            (base_image.extent()[0], base_image.extent()[1])
        };

        // Here we use staging buffer to create optimal image on DEVICE_LOCAL memory.
        // For initialize detail of linear tiling image, please visit
        // https://github.com/SaschaWillems/Vulkan/blob/master/examples/texture/texture.cpp#L159.

        // copy data to an optimal tiled image.
        // this loads the texture data into a host local buffer that is copied to the optimal tiled image on the device.

        let staging_buffer = {

            // create a host-visible staging buffer that contains the raw image data.
            // this buffer will be the data source for copying texture data to the optimal tiled image on the device.

            let staging_ci = BufferCI::new(tex_2d.size() as vkbytes)
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
                data_ptr.copy_from(tex_2d.data() as *const u8, tex_2d.size());
            }

            device.vma.unmap_memory(&staging_allocation.1)
                .map_err(VkErrorKind::Vma)?;

            VmaBuffer::from(staging_allocation)
        };

        // setup buffer copy regions for each mip level.
        let mut buffer_copy_regions = Vec::with_capacity(tex_2d.levels());
        let mut staging_offset = 0;

        for i in 0..tex_2d.levels() {

            let image_level_i = tex_2d.get_level(i);

            let copy_region = vk::BufferImageCopy {
                buffer_offset: staging_offset,
                // specify the following two member to 0 to tell vulkan the image is tightly packed.
                buffer_row_length  : 0,
                buffer_image_height: 0,
                image_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: i as vkuint,
                    base_array_layer: 0,
                    layer_count     : 1,
                },
                image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
                image_extent: vk::Extent3D {
                    width : image_level_i.extent()[0],
                    height: image_level_i.extent()[1],
                    depth : 1,
                },
            };

            buffer_copy_regions.push(copy_region);
            staging_offset += image_level_i.size() as vkbytes;
        }

        // create optimal tiled target image on the device.
        let dst_image = {

            let image_ci = ImageCI::new_2d(format, vk::Extent2D { width, height })
                .mip_levels(tex_2d.levels() as vkuint)
                .array_layers(1)
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


        { // image memory barriers for the texture image.

            // The sub resource range describes the regions of the image that will be transitioned using the memory barriers below.
            let sub_range = vk::ImageSubresourceRange {
                // Image only contains color data.
                aspect_mask: vk::ImageAspectFlags::COLOR,
                // Start at first mip level.
                base_mip_level: 0,
                // We will transition on all mip levels.
                level_count   : tex_2d.levels() as vkuint,
                base_array_layer: 0,
                // The 2D texture only has one layer.
                layer_count     : 1,
            };

            // Transition the texture image layout to transfer target, so we can safely copy our buffer data to it.
            let barrier1 = ImageBarrierCI::new(dst_image.handle, sub_range)
                .access_mask(vk::AccessFlags::empty(), vk::AccessFlags::TRANSFER_WRITE)
                .layout(vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL);

            // Once the data has been uploaded we transfer to the texture image to the shader read layout, so it can be sampled from.
            let barrier2 = ImageBarrierCI::new(dst_image.handle, sub_range)
                .access_mask(vk::AccessFlags::TRANSFER_WRITE, vk::AccessFlags::SHADER_READ)
                .layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

            // prepare vulkan object to transfer data.
            let command_pool = CommandPoolCI::new(device.logic.queues.transfer.family_index)
                .build(device)?;
            let copy_command = CommandBufferAI::new(command_pool, 1)
                .build(device)?
                .remove(0);
            let cmd_recorder: VkCmdRecorder<ITransfer> = VkCmdRecorder::new(&device.logic, copy_command);

            cmd_recorder.begin_record()?
                // Insert a memory dependency at the proper pipeline stages that will execute the image layout transition.
                // Source pipeline stage is host write/read execution (vk::PipelineStageFlags::HOST)
                // Destination pipeline stage is copy command execution (vk::PipelineStageFlags::TRANSFER)
                .image_pipeline_barrier(vk::PipelineStageFlags::HOST, vk::PipelineStageFlags::TRANSFER, vk::DependencyFlags::empty(), &[barrier1.value()])
                // Copy mip levels from staging buffer.
                .copy_buf2img(staging_buffer.handle, dst_image.handle, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &buffer_copy_regions)
                // Insert a memory dependency at the proper pipeline stages that will execute the image layout transition.
                // Source pipeline stage stage is copy command execution (vk::PipelineStageFlags::TRANSFER).
                // Destination pipeline stage fragment shader access (vk::PipelineStageFlags::FRAGMENT_SHADER).
                .image_pipeline_barrier(vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::FRAGMENT_SHADER, vk::DependencyFlags::empty(), &[barrier2.value()])
                .end_record()?;

            cmd_recorder.flush_copy_command(device.logic.queues.transfer.handle)?;

            // free the command poll will automatically destroy all command buffers created by this pool.
            device.discard(command_pool);
        }


        { // clean up staging resources.
            device.vma_discard(&staging_buffer)?;
        }

        let dst_sampler = {

            // Create a texture sampler.
            // In Vulkan textures are accessed by samplers.
            // This separates all the sampling information from the texture data.
            // This means you could have multiple sampler objects for the same texture with different settings.

            let mut sampler_ci = SamplerCI::new()
                .filter(vk::Filter::LINEAR, vk::Filter::LINEAR)
                .mipmap(vk::SamplerMipmapMode::LINEAR, vk::SamplerAddressMode::REPEAT, vk::SamplerAddressMode::REPEAT, vk::SamplerAddressMode::REPEAT)
                // Set max level-of-detail to mip level count of the texture.
                .lod(0.0, 0.0, tex_2d.levels() as vkfloat)
                .compare_op(Some(vk::CompareOp::NEVER))
                .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE);

            // Enable anisotropic filtering.
            // This feature is optional, so we must check if it's supported on the device.
            if device.phy.enable_features().sampler_anisotropy == vk::TRUE {
                // Use max level of anisotropy for this example.
                sampler_ci = sampler_ci.anisotropy(Some(device.phy.limits.max_sampler_anisotropy));
            } else {
                sampler_ci = sampler_ci.anisotropy(None);
            }

            sampler_ci.build(device)?
        };

        let dst_image_view = {

            // Create image view.
            // Textures are not directly accessed by the shaders and are abstracted by image views containing additional.
            // information and sub resource ranges.

            ImageViewCI::new(dst_image.handle, vk::ImageViewType::TYPE_2D, format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::R,
                    g: vk::ComponentSwizzle::G,
                    b: vk::ComponentSwizzle::B,
                    a: vk::ComponentSwizzle::A,
                })
                // The subresource range describes the set of mip levels (and array layers) that can be accessed through this image view.
                // It's possible to create multiple image views for a single image referring to different (and/or overlapping) ranges of the image.
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                // Linear tiling usually won't support mip maps. Only set mip map count if optimal tiling is used.
                .mip_level(0, tex_2d.levels() as vkuint)
                .array_layers(0, 1)
                .build(device)?
        };


        let result = Texture {
            sampler: dst_sampler,
            image  : dst_image,
            view   : dst_image_view,
            layout : vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            mip_levels: tex_2d.levels() as vkuint,
            width, height,
        };
        Ok(result)
    }

    pub fn discard(&self, device: &mut VkDevice) -> VkResult<()> {

        device.discard(self.sampler);
        device.discard(self.view);
        device.vma_discard(&self.image)
    }
}


use ash::vk;

use gli::GliTexture;

use crate::ci::vma::{VmaImage, VmaBuffer, VmaAllocationCI};
use crate::ci::image::{ImageCI, ImageViewCI, ImageBarrierCI, SamplerCI};
use crate::ci::buffer::BufferCI;
use crate::ci::VkObjectBuildableCI;

use crate::command::CmdTransferApi;
use crate::context::VkDevice;

use crate::{VkResult, VkErrorKind};
use crate::{vkuint, vkbytes, vkfloat};

use std::path::Path;
use std::ptr;


/// 2D texture.
pub struct Texture2D {

    pub image: VmaImage,
    pub view : vk::ImageView,

    pub width      : vkuint,
    pub height     : vkuint,
    pub mip_levels : vkuint,

    pub sampler: vk::Sampler,
    pub descriptor: vk::DescriptorImageInfo,
}

impl Texture2D {

    pub fn load_ktx(device: &mut VkDevice, path: impl AsRef<Path>, format: vk::Format) -> VkResult<Texture2D> {

        let tex_2d: gli::Texture2D = gli::load_ktx(path)
            .map_err(VkErrorKind::Gli)?;

        debug_assert!(!tex_2d.empty());

        let (width, height) = {
            let base_image = tex_2d.get_level(0);
            (base_image.extent().width, base_image.extent().height)
        };

        // Only use linear tiling if requested (and supported by the device).
        // Support for linear tiling is mostly limited, so prefer to use optimal tiling instead.
        // On most implementations linear tiling will only support a very limited amount of formats and features (mip maps, cubemap, arrays, etc.).

        let staging_buffer = {

            // create a host-visible staging buffer that contains the raw image data.
            // This buffer is used as a transfer source for the buffer copy.

            let staging_ci = BufferCI::new(tex_2d.size() as vkbytes)
                .usage(vk::BufferUsageFlags::TRANSFER_SRC);
            let allocation_ci = VmaAllocationCI::new(vma::MemoryUsage::CpuOnly, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
            let staging_allocation = device.vma.create_buffer(
                &staging_ci, &allocation_ci)
                .map_err(VkErrorKind::Vma)?;

            // Copy texture data into staging buffer.
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
                    width : image_level_i.extent().width,
                    height: image_level_i.extent().height,
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
                &image_ci, &allocation_ci)
                .map_err(VkErrorKind::Vma)?;

            VmaImage::from(image_allocation)
        };


        { // transfer image data from staging buffer to dst image.

            let sub_range = vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: tex_2d.levels() as vkuint,
                base_array_layer: 0,
                layer_count: 1,
            };

            // Image barrier for optimal image (target).
            // Optimal image will be used as destination for the copy.
            let barrier1 = ImageBarrierCI::new(dst_image.handle, sub_range)
                .access_mask(vk::AccessFlags::empty(), vk::AccessFlags::TRANSFER_WRITE)
                .layout(vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL);

            // Change texture image layout to shader read after all mip levels have been copied.
            let barrier2 = ImageBarrierCI::new(dst_image.handle, sub_range)
                .access_mask(vk::AccessFlags::TRANSFER_WRITE, vk::AccessFlags::SHADER_READ)
                .layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);


            let cmd_recorder = device.get_transfer_recorder();

            cmd_recorder.begin_record()?
                .image_pipeline_barrier(vk::PipelineStageFlags::HOST, vk::PipelineStageFlags::TRANSFER, vk::DependencyFlags::empty(), &[barrier1.value()])
                // Copy mip levels from staging buffer.
                .copy_buf2img(staging_buffer.handle, dst_image.handle, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &buffer_copy_regions)
                .image_pipeline_barrier(vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::ALL_COMMANDS, vk::DependencyFlags::empty(), &[barrier2.value()])
                .end_record()?;

            device.flush_transfer(cmd_recorder)?;
        }


        { // clean up staging resources.
            device.vma_discard(staging_buffer)?;
        }

        let dst_sampler = {

            // Create a default sampler.
            let mut sampler_ci = SamplerCI::new()
                .filter(vk::Filter::LINEAR, vk::Filter::LINEAR)
                .mipmap(vk::SamplerMipmapMode::LINEAR)
                .address(vk::SamplerAddressMode::REPEAT, vk::SamplerAddressMode::REPEAT, vk::SamplerAddressMode::REPEAT)
                // max level-of-detail should match mip level count.
                .lod(0.0, 0.0, tex_2d.levels() as vkfloat)
                .compare_op(Some(vk::CompareOp::NEVER))
                .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE);

            // Only enable anisotropic filtering if enabled on the device.
            sampler_ci = if device.phy.features_enabled().sampler_anisotropy == vk::TRUE {
                sampler_ci.anisotropy(Some(device.phy.limits.max_sampler_anisotropy))
            } else {
                sampler_ci.anisotropy(None)
            };

            sampler_ci.build(device)?
        };

        let dst_image_view = ImageViewCI::new(dst_image.handle, vk::ImageViewType::TYPE_2D, format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::R,
                g: vk::ComponentSwizzle::G,
                b: vk::ComponentSwizzle::B,
                a: vk::ComponentSwizzle::A,
            })
            .sub_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: tex_2d.levels() as vkuint,
                base_array_layer: 0,
                layer_count: 1,
            })
            .build(device)?;


        let result = Texture2D {
            image: dst_image,
            view : dst_image_view,
            mip_levels: tex_2d.levels() as vkuint,
            sampler: dst_sampler,
            descriptor: vk::DescriptorImageInfo {
                sampler: dst_sampler,
                image_view: dst_image_view,
                image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            },
            width, height,
        };
        Ok(result)
    }

    pub fn discard_by(self, device: &mut VkDevice) -> VkResult<()> {

        device.discard(self.sampler);
        device.discard(self.view);
        device.vma_discard(self.image)
    }
}

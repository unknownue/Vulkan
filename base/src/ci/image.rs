
use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::VkDevice;
use crate::ci::VulkanCI;
use crate::error::{VkResult, VkError};
use crate::vkuint;

use std::ptr;

// ----------------------------------------------------------------------------------------------
// Wrapper class for vk::ImageCreateInfo.
#[derive(Debug, Clone)]
pub struct ImageCI {

    ci: vk::ImageCreateInfo,
    queue_families: Vec<vkuint>,
}

impl VulkanCI<vk::ImageCreateInfo> for ImageCI {

    fn inner_default() -> ImageCI {

        ImageCI {
            ci: vk::ImageCreateInfo {
                s_type: vk::StructureType::IMAGE_CREATE_INFO,
                p_next: ptr::null(),
                flags : vk::ImageCreateFlags::empty(),
                image_type: vk::ImageType::TYPE_2D,
                format: vk::Format::UNDEFINED,
                extent: Default::default(),
                mip_levels  : 1,
                array_layers: 1,
                samples: vk::SampleCountFlags::TYPE_1,
                tiling : vk::ImageTiling::OPTIMAL,
                usage  : vk::ImageUsageFlags::empty(),
                sharing_mode  : vk::SharingMode::EXCLUSIVE,
                initial_layout: vk::ImageLayout::UNDEFINED,
                queue_family_index_count: 0,
                p_queue_family_indices  : ptr::null(),
            },
            queue_families: Vec::new(),
        }
    }
}

impl ImageCI {

    pub fn new(r#type: vk::ImageType, format: vk::Format, dimension: vk::Extent3D) -> ImageCI {

        let mut image_ci = ImageCI::inner_default();
        image_ci.ci.image_type = r#type;
        image_ci.ci.format = format;
        image_ci.ci.extent = dimension;

        image_ci
    }

    pub fn new_2d(format: vk::Format, dimension: vk::Extent2D) -> ImageCI {

        let mut image_ci = ImageCI::inner_default();
        image_ci.ci.format = format;
        image_ci.ci.extent = vk::Extent3D {
            width : dimension.width,
            height: dimension.height,
            depth : 1,
        };

        image_ci
    }

    pub fn build(mut self, device: &VkDevice) -> VkResult<(vk::Image, vk::MemoryRequirements)> {

        self.ci.queue_family_index_count = self.queue_families.len() as _;
        self.ci.p_queue_family_indices = self.queue_families.as_ptr();

        let image = unsafe {
            device.logic.handle.create_image(&self.ci, None)
                .map_err(|_| VkError::create("Image"))?
        };

        let requirement = unsafe {
            device.logic.handle.get_image_memory_requirements(image)
        };

        Ok((image, requirement))
    }

    pub fn flags(mut self, flags: vk::ImageCreateFlags) -> ImageCI {
        self.ci.flags = flags; self
    }

    pub fn usages(mut self, flags: vk::ImageUsageFlags) -> ImageCI {
        self.ci.usage = flags; self
    }

    pub fn tiling(mut self, tiling: vk::ImageTiling) -> ImageCI {
        self.ci.tiling = tiling; self
    }

    pub fn samples(mut self, count: vk::SampleCountFlags) -> ImageCI {
        self.ci.samples = count; self
    }

    pub fn mip_levels(mut self, level: vkuint) -> ImageCI {
        self.ci.mip_levels = level; self
    }

    pub fn array_layers(mut self, layers: vkuint) -> ImageCI {
        self.ci.array_layers = layers; self
    }

    pub fn initial_layout(mut self, layout: vk::ImageLayout) -> ImageCI {
        self.ci.initial_layout = layout; self
    }

    pub fn sharing_queues(mut self, mode: vk::SharingMode, families_indices: Vec<vkuint>) -> ImageCI {
        self.queue_families = families_indices;
        self.ci.sharing_mode = mode; self
    }
}

impl crate::context::VulkanObject for vk::Image {

    fn discard(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_image(self, None);
        }
    }
}

impl From<ImageCI> for vk::ImageCreateInfo {

    fn from(value: ImageCI) -> vk::ImageCreateInfo {
        value.ci
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
// Wrapper class for vk::ImageViewCreateInfo.
#[derive(Debug, Clone)]
pub struct ImageViewCI {
    ci: vk::ImageViewCreateInfo,
}

impl VulkanCI<vk::ImageViewCreateInfo> for ImageViewCI {

    fn inner_default() -> ImageViewCI {

        ImageViewCI {
            ci: vk::ImageViewCreateInfo {
                s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                p_next: ptr::null(),
                flags : vk::ImageViewCreateFlags::empty(),
                image : vk::Image::null(),
                view_type: vk::ImageViewType::TYPE_2D,
                format: vk::Format::UNDEFINED,
                components: vk::ComponentMapping {
                    r: vk::ComponentSwizzle::R,
                    g: vk::ComponentSwizzle::G,
                    b: vk::ComponentSwizzle::B,
                    a: vk::ComponentSwizzle::A,
                },
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask      : vk::ImageAspectFlags::COLOR,
                    base_mip_level   : 0,
                    level_count      : 1,
                    base_array_layer : 0,
                    layer_count      : 1,
                },
            },
        }
    }
}

impl ImageViewCI {

    pub fn new(image: vk::Image, r#type: vk::ImageViewType, format: vk::Format) -> ImageViewCI {

        let mut image_view_ci = ImageViewCI::inner_default();
        image_view_ci.ci.image = image;
        image_view_ci.ci.view_type = r#type;
        image_view_ci.ci.format = format;

        image_view_ci
    }

    pub fn build(self, device: &VkDevice) -> VkResult<vk::ImageView> {

        let view = unsafe {
            device.logic.handle.create_image_view(&self.ci, None)
                .map_err(|_| VkError::create("Image View"))?
        };
        Ok(view)
    }

    pub fn flags(mut self, flags: vk::ImageViewCreateFlags) -> ImageViewCI {
        self.ci.flags = flags; self
    }

    pub fn components(mut self, components: vk::ComponentMapping) -> ImageViewCI {
        self.ci.components = components;; self
    }

    pub fn aspect_mask(mut self, aspect: vk::ImageAspectFlags) -> ImageViewCI {
        self.ci.subresource_range.aspect_mask = aspect; self
    }

    pub fn mip_level(mut self, base_level: vkuint, level_count: vkuint) -> ImageViewCI {
        self.ci.subresource_range.base_mip_level = base_level;
        self.ci.subresource_range.level_count = level_count; self
    }

    pub fn array_layers(mut self, base_layer: vkuint, layer_count: vkuint) -> ImageViewCI {
        self.ci.subresource_range.base_array_layer = base_layer;
        self.ci.subresource_range.layer_count = layer_count; self
    }
}

impl crate::context::VulkanObject for vk::ImageView {

    fn discard(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_image_view(self, None)
        }
    }
}

impl From<ImageViewCI> for vk::ImageViewCreateInfo {

    fn from(value: ImageViewCI) -> vk::ImageViewCreateInfo {
        value.ci
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
// Wrapper class for vk::ImageMemoryBarrier.
#[derive(Debug, Clone)]
pub struct ImageBarrierCI {
    ci: vk::ImageMemoryBarrier,
}

impl VulkanCI<vk::ImageMemoryBarrier> for ImageBarrierCI {

    fn inner_default() -> ImageBarrierCI {

        let barrier = vk::ImageMemoryBarrier {
            s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
            p_next: ptr::null(),
            src_access_mask: vk::AccessFlags::empty(),
            dst_access_mask: vk::AccessFlags::empty(),
            old_layout: vk::ImageLayout::UNDEFINED,
            new_layout: vk::ImageLayout::UNDEFINED,
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            image: vk::Image::null(),
            subresource_range: Default::default(),
        };

        ImageBarrierCI { ci: barrier }
    }
}

impl ImageBarrierCI {

    pub fn new(image: vk::Image, subrange: vk::ImageSubresourceRange) -> ImageBarrierCI {

        let mut barrier = ImageBarrierCI::inner_default();
        barrier.ci.image = image;
        barrier.ci.subresource_range = subrange;

        barrier
    }

    pub fn build(&self) -> vk::ImageMemoryBarrier {
        self.ci.clone()
    }

    pub fn access_mask(mut self, from: vk::AccessFlags, to: vk::AccessFlags) -> Self {
        self.ci.src_access_mask = from;
        self.ci.dst_access_mask = to;
        self
    }

    pub fn layout(mut self, from: vk::ImageLayout, to: vk::ImageLayout) -> Self {
        self.ci.old_layout = from;
        self.ci.new_layout = to;
        self
    }

    pub fn queue_family_index(mut self, from: vkuint, to: vkuint) -> Self {
        self.ci.src_queue_family_index = from;
        self.ci.dst_queue_family_index = to;
        self
    }
}

impl From<ImageBarrierCI> for vk::ImageMemoryBarrier {

    fn from(value: ImageBarrierCI) -> vk::ImageMemoryBarrier {
        value.ci
    }
}
// ----------------------------------------------------------------------------------------------
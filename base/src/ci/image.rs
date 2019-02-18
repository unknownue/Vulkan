
use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::{VkDevice, VkObjectCreatable, VkObjectBindable};
use crate::ci::{VulkanCI, VkObjectBuildableCI};
use crate::error::{VkResult, VkError};
use crate::{vkbytes, vkuint};

use std::ptr;

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::ImageCreateInfo.
#[derive(Debug, Clone)]
pub struct ImageCI {

    ci: vk::ImageCreateInfo,
    queue_families: Vec<vkuint>,
}

impl VulkanCI for ImageCI {
    type CIType = vk::ImageCreateInfo;

    fn default_ci() -> Self::CIType {

        vk::ImageCreateInfo {
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
        }
    }
}

impl VkObjectBuildableCI for ImageCI {
    type ObjectType = (vk::Image, vk::MemoryRequirements);

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let image_ci = vk::ImageCreateInfo {
            queue_family_index_count: self.queue_families.len() as _,
            p_queue_family_indices  : self.queue_families.as_ptr(),
            ..self.ci
        };

        let image = unsafe {
            device.logic.handle.create_image(&image_ci, None)
                .map_err(|_| VkError::create("Image"))?
        };

        let requirement = unsafe {
            device.logic.handle.get_image_memory_requirements(image)
        };

        Ok((image, requirement))
    }
}

impl ImageCI {

    pub fn new(r#type: vk::ImageType, format: vk::Format, dimension: vk::Extent3D) -> ImageCI {

        ImageCI {
            ci: vk::ImageCreateInfo {
                image_type: r#type,
                format,
                extent: dimension,
                ..ImageCI::default_ci()
            },
            queue_families: Vec::new(),
        }
    }

    pub fn new_2d(format: vk::Format, dimension: vk::Extent2D) -> ImageCI {

        let extent = vk::Extent3D {
            width : dimension.width,
            height: dimension.height,
            depth : 1,
        };

        ImageCI::new(vk::ImageType::TYPE_2D, format, extent)
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

impl VkObjectCreatable for vk::Image {

    fn discard(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_image(self, None);
        }
    }
}

impl VkObjectBindable for vk::Image {

    fn bind(self, device: &VkDevice, memory: vk::DeviceMemory, offset: vkbytes) -> VkResult<()> {
        unsafe {
            device.logic.handle.bind_image_memory(self, memory, offset)
                .map_err(|_| VkError::device("Binding Image Memory"))
        }
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::ImageViewCreateInfo.
#[derive(Debug, Clone)]
pub struct ImageViewCI {
    ci: vk::ImageViewCreateInfo,
}

impl VulkanCI for ImageViewCI {
    type CIType = vk::ImageViewCreateInfo;

    fn default_ci() -> Self::CIType {

        vk::ImageViewCreateInfo {
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
        }
    }
}

impl VkObjectBuildableCI for ImageViewCI {
    type ObjectType = vk::ImageView;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let view = unsafe {
            device.logic.handle.create_image_view(&self.ci, None)
                .map_err(|_| VkError::create("Image View"))?
        };
        Ok(view)
    }
}

impl ImageViewCI {

    pub fn new(image: vk::Image, r#type: vk::ImageViewType, format: vk::Format) -> ImageViewCI {

        ImageViewCI {
            ci: vk::ImageViewCreateInfo {
                image, format,
                view_type: r#type,
                ..ImageViewCI::default_ci()
            },
        }
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

impl crate::context::VkObjectCreatable for vk::ImageView {

    fn discard(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_image_view(self, None)
        }
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::ImageMemoryBarrier.
#[derive(Debug, Clone)]
pub struct ImageBarrierCI {
    ci: vk::ImageMemoryBarrier,
}

impl VulkanCI for ImageBarrierCI {
    type CIType = vk::ImageMemoryBarrier;

    fn default_ci() -> Self::CIType {

        vk::ImageMemoryBarrier {
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
        }
    }
}

impl ImageBarrierCI {

    pub fn new(image: vk::Image, subrange: vk::ImageSubresourceRange) -> ImageBarrierCI {

        ImageBarrierCI {
            ci: vk::ImageMemoryBarrier {
                image,
                subresource_range: subrange,
                ..ImageBarrierCI::default_ci()
            },
        }
    }

    pub fn value(&self) -> vk::ImageMemoryBarrier {
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

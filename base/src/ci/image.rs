
use ash::vk;

use crate::vkuint;

use std::ptr;

#[derive(Debug, Clone)]
pub struct ImageBarrierCI {
    ci: vk::ImageMemoryBarrier,
}

impl ImageBarrierCI {

    pub fn new(image: vk::Image, subrange: vk::ImageSubresourceRange) -> ImageBarrierCI {

        let mut barrier = ImageBarrierCI::inner_default();
        barrier.ci.image = image;
        barrier.ci.subresource_range = subrange;

        barrier
    }

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

    pub(crate) fn build(self) -> vk::ImageMemoryBarrier {
        self.ci
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

    fn from(v: ImageBarrierCI) -> vk::ImageMemoryBarrier {
        v.ci
    }
}

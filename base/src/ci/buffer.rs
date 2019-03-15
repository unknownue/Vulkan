
use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::{VkDevice, VkObjectDiscardable, VkObjectBindable};
use crate::ci::{VulkanCI, VkObjectBuildableCI};
use crate::error::{VkResult, VkError};
use crate::{vkuint, vkbytes};

use std::ptr;
use std::ops::Deref;

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::BufferCreateInfo.
#[derive(Debug, Clone)]
pub struct BufferCI {
    inner: vk::BufferCreateInfo,
    queue_families: Option<Vec<vkuint>>,
}

impl VulkanCI<vk::BufferCreateInfo> for BufferCI {

    fn default_ci() -> vk::BufferCreateInfo {

        vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            p_next: ptr::null(),
            flags : vk::BufferCreateFlags::empty(),
            size  : 0,
            usage : vk::BufferUsageFlags::empty(),
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices  : ptr::null(),
        }
    }
}

impl Deref for BufferCI {
    type Target = vk::BufferCreateInfo;

    fn deref(&self) -> &vk::BufferCreateInfo {
        &self.inner
    }
}

impl VkObjectBuildableCI for BufferCI {
    type ObjectType = (vk::Buffer, vk::MemoryRequirements);

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let buffer = unsafe {
            device.logic.handle.create_buffer(self, None)
                .map_err(|_| VkError::create("Buffer"))?
        };

        let requirement = unsafe {
            device.logic.handle.get_buffer_memory_requirements(buffer)
        };

        Ok((buffer, requirement))
    }
}

impl BufferCI {

    pub fn new(size: vkbytes) -> BufferCI {

        BufferCI {
            inner: vk::BufferCreateInfo {
                size,
                ..BufferCI::default_ci()
            },
            queue_families: None,
        }
    }

    #[inline(always)]
    pub fn flags(mut self, flags: vk::BufferCreateFlags) -> BufferCI {
        self.inner.flags = flags; self
    }

    #[inline(always)]
    pub fn usage(mut self, flags: vk::BufferUsageFlags) -> BufferCI {
        self.inner.usage = flags; self
    }

    #[inline(always)]
    pub fn sharing_queues(mut self, mode: vk::SharingMode, families_indices: Vec<vkuint>) -> BufferCI {

        self.inner.queue_family_index_count = families_indices.len() as _;
        self.inner.p_queue_family_indices   = families_indices.as_ptr();

        self.queue_families = Some(families_indices);
        self.inner.sharing_mode = mode; self
    }
}

impl VkObjectDiscardable for vk::Buffer {

    fn discard_by(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_buffer(self, None)
        }
    }
}

impl VkObjectBindable for vk::Buffer {

    fn bind(self, device: &VkDevice, memory: vk::DeviceMemory, offset: vkbytes) -> VkResult<()> {
        unsafe {
            device.logic.handle.bind_buffer_memory(self, memory, offset)
                .map_err(|_| VkError::device("Binding Buffer Memory"))
        }
    }
}
// ----------------------------------------------------------------------------------------------

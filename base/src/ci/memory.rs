
use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::VkDevice;
use crate::ci::{VulkanCI, VkObjectBuildableCI};
use crate::error::{VkResult, VkError};
use crate::{vkuint, vkbytes};

use std::ptr;
use std::ops::Deref;

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::MemoryAllocateInfo.
#[derive(Debug, Clone)]
pub struct MemoryAI {
    inner: vk::MemoryAllocateInfo,
}

impl VulkanCI<vk::MemoryAllocateInfo> for MemoryAI {

    fn default_ci() -> vk::MemoryAllocateInfo {

        vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: ptr::null(),
            allocation_size  : 0,
            memory_type_index: 0,
        }
    }
}

impl Deref for MemoryAI {
    type Target = vk::MemoryAllocateInfo;

    fn deref(&self) -> &vk::MemoryAllocateInfo {
        &self.inner
    }
}

impl VkObjectBuildableCI for MemoryAI {
    type ObjectType = vk::DeviceMemory;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let memory = unsafe {
            device.logic.handle.allocate_memory(self, None)
                .map_err(|_| VkError::create("Memory Allocate"))?
        };
        Ok(memory)
    }
}

impl MemoryAI {

    pub fn new(allocation_size: vkbytes, memory_type_index: vkuint) -> MemoryAI {

        MemoryAI {
            inner: vk::MemoryAllocateInfo {
                allocation_size, memory_type_index,
                ..MemoryAI::default_ci()
            },
        }
    }
}

impl crate::context::VkObjectDiscardable for vk::DeviceMemory {

    fn discard_by(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.free_memory(self, None);
        }
    }
}
// ----------------------------------------------------------------------------------------------

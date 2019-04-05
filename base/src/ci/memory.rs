//! Types which simplify the creation of Vulkan memory objects.

use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::VkDevice;
use crate::ci::{VulkanCI, VkObjectBuildableCI};
use crate::error::{VkResult, VkError};
use crate::{vkuint, vkbytes};

use std::ptr;

// ----------------------------------------------------------------------------------------------
/// Wrapper class for `vk::MemoryAllocateInfo`.
///
/// The default values are defined as follows:
/// ``` ignore
/// vk::MemoryAllocateInfo {
///    s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
///    p_next: ptr::null(),
///    allocation_size  : 0,
///    memory_type_index: 0,
/// }
/// ```
///
/// See [VkMemoryAllocateInfo](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkMemoryAllocateInfo.html) for more detail.
///
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

impl AsRef<vk::MemoryAllocateInfo> for MemoryAI {

    fn as_ref(&self) -> &vk::MemoryAllocateInfo {
        &self.inner
    }
}

impl VkObjectBuildableCI for MemoryAI {
    type ObjectType = vk::DeviceMemory;

    /// Allocate `vk::DeviceMemory` object, and return its handle.
    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let memory = unsafe {
            device.logic.handle.allocate_memory(self.as_ref(), None)
                .map_err(|_| VkError::create("Memory Allocate"))?
        };
        Ok(memory)
    }
}

impl MemoryAI {

    /// Initialize `vk::MemoryAllocateInfo` with default value.
    ///
    /// `allocation_size` is the size in bytes of the memory to be allocated.
    ///
    /// `memory_type_index` is the index identifying a memory type querying from Vulkan.
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

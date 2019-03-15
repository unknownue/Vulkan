
use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::VkDevice;
use crate::context::{VkObjectDiscardable, VkObjectAllocatable};
use crate::ci::{VulkanCI, VkObjectBuildableCI};
use crate::error::{VkResult, VkError};
use crate::vkuint;

use std::ptr;
use std::ops::Deref;

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::CommandBufferAllocateInfo.
#[derive(Debug, Clone)]
pub struct CommandBufferAI {
    inner: vk::CommandBufferAllocateInfo,
}

impl VulkanCI<vk::CommandBufferAllocateInfo> for CommandBufferAI {

    fn default_ci() -> vk::CommandBufferAllocateInfo {

        vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_pool: vk::CommandPool::null(),
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: 1,
        }
    }
}

impl Deref for CommandBufferAI {
    type Target = vk::CommandBufferAllocateInfo;

    fn deref(&self) -> &vk::CommandBufferAllocateInfo {
        &self.inner
    }
}


impl VkObjectBuildableCI for CommandBufferAI {
    type ObjectType = Vec<vk::CommandBuffer>;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let commands = unsafe {
            device.logic.handle.allocate_command_buffers(self)
                .map_err(|_| VkError::create("Command Buffers"))?
        };
        Ok(commands)
    }
}

impl CommandBufferAI {

    pub fn new(pool: vk::CommandPool, count: vkuint) -> CommandBufferAI {

        CommandBufferAI {
            inner: vk::CommandBufferAllocateInfo {
                command_pool: pool,
                command_buffer_count: count,
                ..CommandBufferAI::default_ci()
            }
        }
    }

    #[inline(always)]
    pub fn level(mut self, level: vk::CommandBufferLevel) -> CommandBufferAI {
        self.inner.level = level; self
    }
}

impl VkObjectAllocatable for vk::CommandBuffer {
    type AllocatePool = vk::CommandPool;

    fn free(self, device: &VkDevice, pool: Self::AllocatePool) {
        unsafe {
            device.logic.handle.free_command_buffers(pool, &[self]);
        }
    }
}

impl VkObjectAllocatable for &[vk::CommandBuffer] {
    type AllocatePool = vk::CommandPool;

    fn free(self, device: &VkDevice, pool: Self::AllocatePool) {
        unsafe {
            device.logic.handle.free_command_buffers(pool, self);
        }
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct CommandPoolCI {
    inner: vk::CommandPoolCreateInfo,
}

impl VulkanCI<vk::CommandPoolCreateInfo> for CommandPoolCI {

    fn default_ci() -> vk::CommandPoolCreateInfo {

        vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags : vk::CommandPoolCreateFlags::empty(),
            queue_family_index: 0,
        }
    }
}

impl Deref for CommandPoolCI {
    type Target = vk::CommandPoolCreateInfo;

    fn deref(&self) -> &vk::CommandPoolCreateInfo {
        &self.inner
    }
}


impl VkObjectBuildableCI for CommandPoolCI {
    type ObjectType = vk::CommandPool;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let pool = unsafe {
            device.logic.handle.create_command_pool(self, None)
                .map_err(|_| VkError::create("Command Pool"))?
        };
        Ok(pool)
    }
}

impl CommandPoolCI {

    pub fn new(queue_family: vkuint) -> CommandPoolCI {

        CommandPoolCI {
            inner: vk::CommandPoolCreateInfo {
                queue_family_index: queue_family,
                ..CommandPoolCI::default_ci()
            },
        }
    }

    #[inline(always)]
    pub fn flags(mut self, flags: vk::CommandPoolCreateFlags) -> CommandPoolCI {
        self.inner.flags = flags; self
    }
}

impl VkObjectDiscardable for vk::CommandPool {

    fn discard_by(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_command_pool(self, None);
        }
    }
}
// ----------------------------------------------------------------------------------------------

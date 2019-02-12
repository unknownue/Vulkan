
use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::VkDevice;
use crate::context::{VkObjectCreatable, VkObjectAllocatable};
use crate::ci::{VulkanCI, VkObjectBuildableCI};
use crate::error::{VkResult, VkError};
use crate::vkuint;

use std::ptr;

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::CommandBufferAllocateInfo.
#[derive(Debug, Clone)]
pub struct CommandBufferAI {
    ai: vk::CommandBufferAllocateInfo,
}

impl VulkanCI for CommandBufferAI {
    type CIType = vk::CommandBufferAllocateInfo;

    fn default_ci() -> Self::CIType {

        vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_pool: vk::CommandPool::null(),
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: 1,
        }
    }
}

impl VkObjectBuildableCI for CommandBufferAI {
    type ObjectType = Vec<vk::CommandBuffer>;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let commands = unsafe {
            device.logic.handle.allocate_command_buffers(&self.ai)
                .map_err(|_| VkError::create("Command Buffers"))?
        };
        Ok(commands)
    }
}

impl CommandBufferAI {

    pub fn new(pool: vk::CommandPool, count: vkuint) -> CommandBufferAI {

        CommandBufferAI {
            ai: vk::CommandBufferAllocateInfo {
                command_pool: pool,
                command_buffer_count: count,
                ..CommandBufferAI::default_ci()
            }
        }
    }

    pub fn level(mut self, level: vk::CommandBufferLevel) -> CommandBufferAI {
        self.ai.level = level; self
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
    ci: vk::CommandPoolCreateInfo,
}

impl VulkanCI for CommandPoolCI {
    type CIType = vk::CommandPoolCreateInfo;

    fn default_ci() -> Self::CIType {

        vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags : vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index: 0,
        }
    }
}

impl VkObjectBuildableCI for CommandPoolCI {
    type ObjectType = vk::CommandPool;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let pool = unsafe {
            device.logic.handle.create_command_pool(&self.ci, None)
                .map_err(|_| VkError::create("Command Pool"))?
        };
        Ok(pool)
    }
}

impl CommandPoolCI {

    pub fn new(queue_family: vkuint) -> CommandPoolCI {

        CommandPoolCI {
            ci: vk::CommandPoolCreateInfo {
                queue_family_index: queue_family,
                ..CommandPoolCI::default_ci()
            },
        }
    }

    pub fn flags(mut self, flags: vk::CommandPoolCreateFlags) -> CommandPoolCI {
        self.ci.flags = flags; self
    }
}

impl VkObjectCreatable for vk::CommandPool {

    fn discard(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_command_pool(self, None);
        }
    }
}
// ----------------------------------------------------------------------------------------------

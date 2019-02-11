
use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::VkDevice;
use crate::context::VulkanObject;
use crate::ci::VulkanCI;
use crate::error::{VkResult, VkError};
use crate::vkuint;

use std::ptr;

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::CommandBufferAllocateInfo.
#[derive(Debug, Clone)]
pub struct CommandBufferAI {
    ai: vk::CommandBufferAllocateInfo,
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

    pub fn build(&self, device: &VkDevice) -> VkResult<Vec<vk::CommandBuffer>> {

        let commands = unsafe {
            device.logic.handle.allocate_command_buffers(&self.ai)
                .map_err(|_| VkError::create("Command Buffers"))?
        };
        Ok(commands)
    }
}

impl From<CommandBufferAI> for vk::CommandBufferAllocateInfo {

    fn from(value: CommandBufferAI) -> vk::CommandBufferAllocateInfo {
        value.ai
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct CommandPoolCI {
    ci: vk::CommandPoolCreateInfo,
}

impl VulkanCI<vk::CommandPoolCreateInfo> for CommandPoolCI {

    fn default_ci() -> vk::CommandPoolCreateInfo {

        vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags : vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index: 0,
        }
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

    pub fn build(&self, device: &VkDevice) -> VkResult<vk::CommandPool> {

        let pool = unsafe {
            device.logic.handle.create_command_pool(&self.ci, None)
                .map_err(|_| VkError::create("Command Pool"))?
        };
        Ok(pool)
    }
}

impl VulkanObject for vk::CommandPool {

    fn discard(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_command_pool(self, None);
        }
    }
}

impl From<CommandPoolCI> for vk::CommandPoolCreateInfo {

    fn from(value: CommandPoolCI) -> vk::CommandPoolCreateInfo {
        value.ci
    }
}
// ----------------------------------------------------------------------------------------------

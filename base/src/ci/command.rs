//! Types which simplify the creation of Vulkan command objects.

use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::VkDevice;
use crate::context::{VkObjectDiscardable, VkObjectAllocatable};
use crate::ci::{VulkanCI, VkObjectBuildableCI};
use crate::error::{VkResult, VkError};
use crate::vkuint;

use std::ptr;

// ----------------------------------------------------------------------------------------------
/// Wrapper class for `vk::CommandBufferAllocateInfo`.
///
/// The default values are defined as follows:
/// ``` ignore
/// vk::CommandBufferAllocateInfo {
///     s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
///     p_next: ptr::null(),
///     command_pool: vk::CommandPool::null(),
///     level: vk::CommandBufferLevel::PRIMARY,
///     command_buffer_count: 1,
/// }
/// ```
///
/// See [VkCommandBufferAllocateInfo](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkCommandBufferAllocateInfo.html) for more detail.
///
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

impl AsRef<vk::CommandBufferAllocateInfo> for CommandBufferAI {

    fn as_ref(&self) -> &vk::CommandBufferAllocateInfo {
        &self.inner
    }
}

impl VkObjectBuildableCI for CommandBufferAI {
    type ObjectType = Vec<vk::CommandBuffer>;

    /// Create `vk::CommandBuffer` objects, and return their handles.
    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let commands = unsafe {
            device.logic.handle.allocate_command_buffers(self.as_ref())
                .map_err(|_| VkError::create("Command Buffers"))?
        };
        Ok(commands)
    }
}

impl CommandBufferAI {

    /// Initialize `vk::CommandBufferAllocateInfo` with default value.
    ///
    /// `pool` is the command pool where command buffers are allocated.
    ///
    /// `count` is the number of command buffers to allocate.
    pub fn new(pool: vk::CommandPool, count: vkuint) -> CommandBufferAI {

        debug_assert!(count > 0, "Command buffer count must be greater than 0!");

        CommandBufferAI {
            inner: vk::CommandBufferAllocateInfo {
                command_pool: pool,
                command_buffer_count: count,
                ..CommandBufferAI::default_ci()
            }
        }
    }

    /// Set the `level` member for `vk::CommandBufferAllocateInfo`.
    ///
    /// It specifies the command buffer level.
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
/// Wrapper class for `vk::CommandPoolCreateInfo`.
///
/// The default values are defined as follows:
/// ``` ignore
/// vk::CommandPoolCreateInfo {
///     s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
///     p_next: ptr::null(),
///     flags : vk::CommandPoolCreateFlags::empty(),
///     queue_family_index: 0,
/// }
/// ```
///
/// See [VkCommandPoolCreateInfo](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkCommandPoolCreateInfo.html) for more detail.
///
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

impl AsRef<vk::CommandPoolCreateInfo> for CommandPoolCI {

    fn as_ref(&self) -> &vk::CommandPoolCreateInfo {
        &self.inner
    }
}

impl VkObjectBuildableCI for CommandPoolCI {
    type ObjectType = vk::CommandPool;

    /// Create `vk::CommandPool` object, and return its handle.
    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let pool = unsafe {
            device.logic.handle.create_command_pool(self.as_ref(), None)
                .map_err(|_| VkError::create("Command Pool"))?
        };
        Ok(pool)
    }
}

impl CommandPoolCI {

    /// Initialize `vk::CommandPoolCreateInfo` with default value.
    ///
    /// `queue_family` is queue family that the command buffers allocated by this command pool are submitted to.
    pub fn new(queue_family: vkuint) -> CommandPoolCI {

        CommandPoolCI {
            inner: vk::CommandPoolCreateInfo {
                queue_family_index: queue_family,
                ..CommandPoolCI::default_ci()
            },
        }
    }

    /// Set the `flags` member for `vk::CommandPoolCreateInfo`.
    ///
    /// It specifies the usage of `vk::CommandPool`.
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

//! Types which simplify some device operations.

use ash::vk;
use ash::version::DeviceV1_0;

use crate::ci::VulkanCI;
use crate::context::{VkSubmitCI, VkDevice};
use crate::error::{VkResult, VkError};

use std::ptr;

// ----------------------------------------------------------------------------------------------
/// Wrapper class for `vk::SubmitInfo`.
///
///
/// The default values are defined as follows:
/// ``` ignore
/// vk::SubmitInfo {
///     s_type: vk::StructureType::SUBMIT_INFO,
///     p_next: ptr::null(),
///     wait_semaphore_count   : 0,
///     p_wait_semaphores      : ptr::null(),
///     p_wait_dst_stage_mask  : ptr::null(),
///     command_buffer_count   : 0,
///     p_command_buffers      : ptr::null(),
///     signal_semaphore_count : 0,
///     p_signal_semaphores    : ptr::null(),
/// }
/// ```
///
/// See [VkSubmitInfo](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkSubmitInfo.html) for more detail.
///
#[derive(Debug, Clone)]
pub struct SubmitCI {

    inner: vk::SubmitInfo,
    wait_stage        : Option<Vec<vk::PipelineStageFlags>>,
    wait_semaphores   : Option<Vec<vk::Semaphore>>,
    signal_semaphores : Option<Vec<vk::Semaphore>>,
    commands          : Vec<vk::CommandBuffer>,
}

impl VulkanCI<vk::SubmitInfo> for SubmitCI {

    fn default_ci() -> vk::SubmitInfo {

        vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count   : 0,
            p_wait_semaphores      : ptr::null(),
            p_wait_dst_stage_mask  : ptr::null(),
            command_buffer_count   : 0,
            p_command_buffers      : ptr::null(),
            signal_semaphore_count : 0,
            p_signal_semaphores    : ptr::null(),
        }
    }
}

impl AsRef<vk::SubmitInfo> for SubmitCI {

    fn as_ref(&self) -> &vk::SubmitInfo {
        &self.inner
    }
}

impl SubmitCI {

    /// Initialize `vk::SubmitInfo` with default value.
    pub fn new() -> SubmitCI {

        SubmitCI {
            inner: SubmitCI::default_ci(),
            wait_stage        : None,
            wait_semaphores   : None,
            signal_semaphores : None,
            commands          : Vec::new(),
        }
    }

    /// Add command buffer to this submit.
    #[inline]
    pub fn add_command(mut self, command: vk::CommandBuffer) -> SubmitCI {

        self.commands.push(command);
        self.inner.command_buffer_count = self.commands.len() as _;
        self.inner.p_command_buffers    = self.commands.as_ptr(); self
    }

    /// Add semaphore to wait before executing the command buffers.
    ///
    /// `semaphore` is the semaphore to wait.
    ///
    /// `stage` is the corresponding pipeline stage for the semaphore.
    #[inline]
    pub fn add_wait(mut self, stage: vk::PipelineStageFlags, semaphore: vk::Semaphore) -> SubmitCI {

        let wait_stages = self.wait_stage.get_or_insert(Vec::new());
        wait_stages.push(stage);

        let wait_semaphores = self.wait_semaphores.get_or_insert(Vec::new());
        wait_semaphores.push(semaphore);

        self.inner.p_wait_dst_stage_mask = wait_stages.as_ptr();
        self.inner.p_wait_semaphores     = wait_semaphores.as_ptr();
        self.inner.wait_semaphore_count  = wait_semaphores.len() as _; self
    }

    /// Add semaphore to be signaled after the executions of command buffers.
    ///
    /// `semaphore` is the semaphore wait to be signaled.
    #[inline]
    pub fn add_signal(mut self, semaphore: vk::Semaphore) -> SubmitCI {

        let signals = self.signal_semaphores.get_or_insert(Vec::new());
        signals.push(semaphore);

        self.inner.signal_semaphore_count = signals.len() as _;
        self.inner.p_signal_semaphores    = signals.as_ptr() as _; self
    }
}

impl VkSubmitCI for vk::SubmitInfo {

    /// Submit the command buffers to specific queue.
    ///
    /// `queue` is the queue that the command buffers will be submitted to.
    ///
    /// `wait_fence` is an optional fence to be signaled after the executions of command buffers.
    fn submit(self, device: &VkDevice, queue: vk::Queue, wait_fence: Option<vk::Fence>) -> VkResult<()> {

        unsafe {
            device.logic.handle.queue_submit(queue, &[self], wait_fence.unwrap_or(vk::Fence::null()))
                .map_err(|_| VkError::device("Queue Submit"))
        }
    }
}

impl VkSubmitCI for SubmitCI {

    fn submit(self, device: &VkDevice, queue: vk::Queue, wait_fence: Option<vk::Fence>) -> VkResult<()> {

        (self.as_ref()).submit(device, queue, wait_fence)
    }
}
// ----------------------------------------------------------------------------------------------

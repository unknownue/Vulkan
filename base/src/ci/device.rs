
use ash::vk;
use ash::version::DeviceV1_0;
use std::ptr;

use crate::ci::VulkanCI;
use crate::context::{VkSubmitCI, VkDevice};
use crate::error::{VkResult, VkError};

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::SubmitInfo.
#[derive(Debug, Clone)]
pub struct SubmitCI {
    ci: vk::SubmitInfo,
    wait_stage        : Vec<vk::PipelineStageFlags>,
    wait_semaphores   : Vec<vk::Semaphore>,
    signal_semaphores : Vec<vk::Semaphore>,
    commands          : Vec<vk::CommandBuffer>,
}

impl VulkanCI for SubmitCI {
    type CIType = vk::SubmitInfo;

    fn default_ci() -> Self::CIType {

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

impl SubmitCI {

    pub fn new() -> SubmitCI {
        SubmitCI {
            ci: SubmitCI::default_ci(),
            wait_stage        : Vec::new(),
            wait_semaphores   : Vec::new(),
            signal_semaphores : Vec::new(),
            commands          : Vec::new(),
        }
    }

    #[inline]
    pub fn add_command(mut self, command: vk::CommandBuffer) -> SubmitCI {
        self.commands.push(command); self
    }

    #[inline]
    pub fn add_wait(mut self, stage: vk::PipelineStageFlags, semaphore: vk::Semaphore) -> SubmitCI {
        self.wait_stage.push(stage);
        self.wait_semaphores.push(semaphore); self
    }

    #[inline]
    pub fn add_signal(mut self, semaphore: vk::Semaphore) -> SubmitCI {
        self.signal_semaphores.push(semaphore); self
    }

    pub fn value(&self) -> vk::SubmitInfo {

        vk::SubmitInfo {
            wait_semaphore_count   : self.wait_semaphores.len() as _,
            p_wait_semaphores      : self.wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask  : self.wait_stage.as_ptr(),
            command_buffer_count   : self.commands.len() as _,
            p_command_buffers      : self.commands.as_ptr(),
            signal_semaphore_count : self.signal_semaphores.len() as _,
            p_signal_semaphores    : self.signal_semaphores.as_ptr(),
            ..self.ci
        }
    }
}

impl VkSubmitCI for vk::SubmitInfo {

    fn submit(self, device: &VkDevice, queue: vk::Queue, wait_fence: vk::Fence) -> VkResult<()> {
        unsafe {
            device.logic.handle.queue_submit(queue, &[self], wait_fence)
                .map_err(|_| VkError::device("Queue Submit"))
        }
    }
}

impl VkSubmitCI for SubmitCI {

    fn submit(self, device: &VkDevice, queue: vk::Queue, wait_fence: vk::Fence) -> VkResult<()> {
        self.value().submit(device, queue, wait_fence)
    }
}
// ----------------------------------------------------------------------------------------------

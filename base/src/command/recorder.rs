
use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::VkDevice;
use crate::error::{VkResult, VkError};

use std::marker::PhantomData;
use std::ptr;

pub struct VkCmdRecorder<'a, T> {

    phantom_marker: PhantomData<T>,

    pub(super) device: &'a VkDevice,
    pub(super) command: vk::CommandBuffer,
    usage  : vk::CommandBufferUsageFlags,
}

impl<'a, 'd: 'a, T> VkCmdRecorder<'a, T> {

    pub fn new(device: &'d VkDevice, command: vk::CommandBuffer) -> VkCmdRecorder<'a, T> {

        VkCmdRecorder {
            device, command,
            usage: vk::CommandBufferUsageFlags::empty(),
            phantom_marker: PhantomData,
        }
    }

    pub fn set_usage(&mut self, flags: vk::CommandBufferUsageFlags) {
        self.usage = flags;
    }

    pub fn begin_record(&self) -> VkResult<&VkCmdRecorder<T>> {

        let begin_ci = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            flags : self.usage,
            p_inheritance_info: ptr::null(),
        };

        unsafe {
            self.device.logic.handle.begin_command_buffer(self.command, &begin_ci)
                .or(Err(VkError::device("Begin Command Buffer.")))?;
        }
        Ok(self)
    }

    pub fn end_record(&self) -> VkResult<()> {

        unsafe {
            self.device.logic.handle.end_command_buffer(self.command)
                .or(Err(VkError::device("End Command Buffer.")))?;
        }

        Ok(())
    }

    pub fn reset_command(&self, flags: vk::CommandBufferResetFlags) -> VkResult<()> {

        unsafe {
            self.device.logic.handle.reset_command_buffer(self.command, flags)
                .or(Err(VkError::device("End Command Buffer.")))?;
        }
        Ok(())
    }
}

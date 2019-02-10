
use ash::vk;

use crate::command::VkCommandType;
use crate::command::recorder::VkCmdRecorder;

pub struct ICompute;

impl VkCommandType for ICompute {
    const BIND_POINT: vk::PipelineBindPoint = vk::PipelineBindPoint::COMPUTE;
}

impl<'a> CmdComputeApi for VkCmdRecorder<'a, ICompute> {
    // Not implement yet...
}

pub trait CmdComputeApi {
    // Not implement yet...
}

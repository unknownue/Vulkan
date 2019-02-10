
mod recorder;
mod graphics;
mod compute;
mod transfer;

use ash::vk;

pub trait VkCommandType {
    const BIND_POINT: vk::PipelineBindPoint;
}

struct IGraphics;

impl VkCommandType for IGraphics {
    const BIND_POINT: vk::PipelineBindPoint = vk::PipelineBindPoint::GRAPHICS;
}


struct ICompute;

impl VkCommandType for ICompute {
    const BIND_POINT: vk::PipelineBindPoint = vk::PipelineBindPoint::COMPUTE;
}


struct ITransfer;

impl VkCommandType for ITransfer {
    const BIND_POINT: vk::PipelineBindPoint = vk::PipelineBindPoint::GRAPHICS;
}

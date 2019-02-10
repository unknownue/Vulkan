
mod recorder;
mod graphics;
mod compute;
mod transfer;

use ash::vk;

pub trait VkCommandType {
    const BIND_POINT: vk::PipelineBindPoint;
}

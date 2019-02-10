
pub use self::recorder::VkCmdRecorder;
pub use self::graphics::{IGraphics, CmdGraphicsApi};
pub use self::compute::{ICompute, CmdComputeApi};
pub use self::transfer::{ITransfer, CmdTransferApi};

mod recorder;
mod graphics;
mod compute;
mod transfer;

use ash::vk;

pub trait VkCommandType {
    const BIND_POINT: vk::PipelineBindPoint;
}

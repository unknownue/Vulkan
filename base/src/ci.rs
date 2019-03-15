
pub mod shader;
pub mod pipeline;
pub mod device;
pub mod image;
pub mod buffer;
pub mod vma;
pub mod descriptor;
pub mod memory;
pub mod command;
pub mod sync;


use crate::context::VkDevice;
use crate::VkResult;

pub(crate) trait VulkanCI<CI>: Sized + AsRef<CI> {

    fn default_ci() -> CI;
}

pub trait VkObjectBuildableCI {
    type ObjectType;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType>;
}

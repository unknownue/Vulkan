
pub mod shader;
pub mod pipeline;
pub mod device;
pub mod image;
pub mod buffer;
pub mod descriptor;
pub mod memory;
pub mod command;
pub mod sync;


use crate::context::VkDevice;
use crate::VkResult;

pub trait VulkanCI
    where
        Self: Sized {
    type CIType;

    fn default_ci() -> Self::CIType;
}

pub trait VkObjectBuildableCI: VulkanCI {
    type ObjectType;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType>;
}

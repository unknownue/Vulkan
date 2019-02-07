
mod physical;
mod logical;
mod queue;

pub use self::logical::{VkLogicalDevice, VkQueue, LogicDevConfig};
pub use self::physical::{VkPhysicalDevice, PhysicalDevConfig};

pub struct VkDevice {

    pub logic : logical::VkLogicalDevice,
    pub phy   : physical::VkPhysicalDevice,
}

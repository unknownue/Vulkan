
mod physical;
mod logical;
mod queue;

pub use self::logical::VkQueue;

pub struct VkDevice {

    pub logic : logical::VkLogicalDevice,
    pub phy   : physical::VkPhysicalDevice,
}


mod physical;
mod logical;
mod queue;

pub struct VkDevice {

    pub logic : logical::VkLogicalDevice,
    pub phy   : physical::VkPhysicalDevice,
}

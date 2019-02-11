
use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::VkDevice;
use crate::ci::VulkanCI;
use crate::error::{VkResult, VkError};

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::SemaphoreCreateInfo.
pub struct SemaphoreCI {
    ci: vk::SemaphoreCreateInfo,
}

//impl VulkanCI<vk::SemaphoreCreateInfo> for SemaphoreCI {
//
//    fn default_ci() -> vk::SemaphoreCreateInfo {
//        unimplemented!()
//    }
//}
// ----------------------------------------------------------------------------------------------

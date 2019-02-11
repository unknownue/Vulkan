
use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::VkDevice;
use crate::context::VulkanObject;
use crate::ci::VulkanCI;
use crate::error::{VkResult, VkError};

use std::ptr;

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::SemaphoreCreateInfo.
#[derive(Debug, Clone)]
pub struct SemaphoreCI {
    ci: vk::SemaphoreCreateInfo,
}

impl VulkanCI<vk::SemaphoreCreateInfo> for SemaphoreCI {

    fn default_ci() -> vk::SemaphoreCreateInfo {

        vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            p_next: ptr::null(),
            flags : vk::SemaphoreCreateFlags::empty(),
        }
    }
}

impl SemaphoreCI {

    pub fn new() -> SemaphoreCI {

        SemaphoreCI {
            ci: SemaphoreCI::default_ci(),
        }
    }

    pub fn flags(mut self, flags: vk::SemaphoreCreateFlags) {
        self.ci.flags = flags;
    }

    pub fn build(&self, device: &VkDevice) -> VkResult<vk::Semaphore> {

        let semaphore = unsafe {
            device.logic.handle.create_semaphore(&self.ci, None)
                .map_err(|_| VkError::create("Semaphore"))?
        };
        Ok(semaphore)
    }
}

impl VulkanObject for vk::Semaphore {

    fn discard(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_semaphore(self, None);
        }
    }
}

impl From<SemaphoreCI> for vk::SemaphoreCreateInfo {

    fn from(value: SemaphoreCI) -> vk::SemaphoreCreateInfo {
        value.ci
    }
}

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::SemaphoreCreateInfo.
#[derive(Debug, Clone)]
pub struct FenceCI {
    ci: vk::FenceCreateInfo,
}

impl VulkanCI<vk::FenceCreateInfo> for FenceCI {

    fn default_ci() -> vk::FenceCreateInfo {

        vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags : vk::FenceCreateFlags::empty(),
        }
    }
}

impl FenceCI {

    pub fn new(is_signed: bool) -> FenceCI {

        let mut fence = FenceCI { ci: FenceCI::default_ci() };

        if is_signed {
            fence.ci.flags = vk::FenceCreateFlags::SIGNALED;
        }

        fence
    }

    pub fn build(&self, device: &VkDevice) -> VkResult<vk::Fence> {

        let fence = unsafe {
            device.logic.handle.create_fence(&self.ci, None)
                .or(Err(VkError::create("Fence")))?
        };
        Ok(fence)
    }
}

impl VulkanObject for vk::Fence {

    fn discard(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_fence(self, None);
        }
    }
}

impl From<FenceCI> for vk::FenceCreateInfo {

    fn from(value: FenceCI) -> vk::FenceCreateInfo {
        value.ci
    }
}
// ----------------------------------------------------------------------------------------------

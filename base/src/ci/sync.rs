
use ash::vk;
use ash::version::DeviceV1_0;

use std::ptr;

use crate::context::VkDevice;
use crate::context::{VkObjectDiscardable, VkObjectWaitable};
use crate::ci::{VulkanCI, VkObjectBuildableCI};
use crate::error::{VkResult, VkError};
use crate::utils::time::VkTimeDuration;


// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::SemaphoreCreateInfo.
#[derive(Debug, Clone)]
pub struct SemaphoreCI {
    ci: vk::SemaphoreCreateInfo,
}

impl VulkanCI for SemaphoreCI {
    type CIType = vk::SemaphoreCreateInfo;

    fn default_ci() -> Self::CIType {

        vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            p_next: ptr::null(),
            flags : vk::SemaphoreCreateFlags::empty(),
        }
    }
}

impl VkObjectBuildableCI for SemaphoreCI {
    type ObjectType = vk::Semaphore;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let semaphore = unsafe {
            device.logic.handle.create_semaphore(&self.ci, None)
                .map_err(|_| VkError::create("Semaphore"))?
        };
        Ok(semaphore)
    }
}

impl SemaphoreCI {

    pub fn new() -> SemaphoreCI {

        SemaphoreCI {
            ci: SemaphoreCI::default_ci(),
        }
    }

    #[inline(always)]
    pub fn flags(mut self, flags: vk::SemaphoreCreateFlags) {
        self.ci.flags = flags;
    }
}

impl VkObjectDiscardable for vk::Semaphore {

    fn discard(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_semaphore(self, None);
        }
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::SemaphoreCreateInfo.
#[derive(Debug, Clone)]
pub struct FenceCI {
    ci: vk::FenceCreateInfo,
}

impl VulkanCI for FenceCI {
    type CIType = vk::FenceCreateInfo;

    fn default_ci() -> Self::CIType {

        vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags : vk::FenceCreateFlags::empty(),
        }
    }
}

impl VkObjectBuildableCI for FenceCI {
    type ObjectType = vk::Fence;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let fence = unsafe {
            device.logic.handle.create_fence(&self.ci, None)
                .or(Err(VkError::create("Fence")))?
        };
        Ok(fence)
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
}

impl VkObjectDiscardable for vk::Fence {

    fn discard(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_fence(self, None);
        }
    }
}

impl VkObjectDiscardable for &Vec<vk::Fence> {

    fn discard(self, device: &VkDevice) {

        for fence in self {
            device.discard(*fence);
        }
    }
}

impl VkObjectWaitable for vk::Fence {

    fn wait(self, device: &VkDevice, time: VkTimeDuration) -> VkResult<()> {
        unsafe {
            device.logic.handle.wait_for_fences(&[self], true, time.into())
                .map_err(|_| VkError::device("Wait for fences"))
        }
    }
}

impl AsRef<vk::FenceCreateInfo> for FenceCI {

    fn as_ref(&self) -> &vk::FenceCreateInfo {
        &self.ci
    }
}
// ----------------------------------------------------------------------------------------------

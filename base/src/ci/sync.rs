//! Types which simplify the creation of Vulkan sync objects.

use ash::vk;
use ash::version::DeviceV1_0;

use std::ptr;

use crate::context::VkDevice;
use crate::context::{VkObjectDiscardable, VkObjectWaitable};
use crate::ci::{VulkanCI, VkObjectBuildableCI};
use crate::error::{VkResult, VkError};
use crate::utils::time::VkTimeDuration;


// ----------------------------------------------------------------------------------------------
/// Wrapper class for `vk::SemaphoreCreateInfo`.
///
/// The default values are defined as follows:
/// ``` ignore
/// vk::SemaphoreCreateInfo {
///     s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
///     p_next: ptr::null(),
///     flags : vk::SemaphoreCreateFlags::empty(),
/// }
/// ```
///
/// See [VkSemaphoreCreateInfo](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkSemaphoreCreateInfo.html) for more detail.
///
#[derive(Debug, Clone)]
pub struct SemaphoreCI {
    inner: vk::SemaphoreCreateInfo,
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

impl AsRef<vk::SemaphoreCreateInfo> for SemaphoreCI {

    fn as_ref(&self) -> &vk::SemaphoreCreateInfo {
        &self.inner
    }
}

impl VkObjectBuildableCI for SemaphoreCI {
    type ObjectType = vk::Semaphore;

    /// Create `vk::Semaphore` object, and return its handle.
    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let semaphore = unsafe {
            device.logic.handle.create_semaphore(self.as_ref(), None)
                .map_err(|_| VkError::create("Semaphore"))?
        };
        Ok(semaphore)
    }
}

impl SemaphoreCI {

    /// Initialize `vk::SemaphoreCreateInfo` with default value.
    #[inline(always)]
    pub fn new() -> SemaphoreCI {

        SemaphoreCI {
            inner: SemaphoreCI::default_ci(),
        }
    }

    /// Set the `flags` member for `vk::SemaphoreCreateInfo`.
    ///
    /// It is still reserved for future use.
    #[inline(always)]
    pub fn flags(mut self, flags: vk::SemaphoreCreateFlags) {
        self.inner.flags = flags;
    }
}

impl VkObjectDiscardable for vk::Semaphore {

    fn discard_by(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_semaphore(self, None);
        }
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
/// Wrapper class for `vk::FenceCreateInfo`.
///
/// The default values are defined as follows:
/// ``` ignore
/// vk::FenceCreateInfo {
///     s_type: vk::StructureType::FENCE_CREATE_INFO,
///     p_next: ptr::null(),
///     flags : vk::FenceCreateFlags::empty(),
/// }
/// ```
///
/// See [VkFenceCreateInfo](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkFenceCreateInfo.html) for more detail.
///
#[derive(Debug, Clone)]
pub struct FenceCI {
    inner: vk::FenceCreateInfo,
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

impl AsRef<vk::FenceCreateInfo> for FenceCI {

    fn as_ref(&self) -> &vk::FenceCreateInfo {
        &self.inner
    }
}

impl VkObjectBuildableCI for FenceCI {
    type ObjectType = vk::Fence;

    /// Create `vk::Fence` object, and return its handle.
    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let fence = unsafe {
            device.logic.handle.create_fence(self.as_ref(), None)
                .or(Err(VkError::create("Fence")))?
        };
        Ok(fence)
    }
}

impl FenceCI {

    /// Initialize `vk::FenceCreateInfo` with default value.
    ///
    /// if `is_signed` is true, `vk::FenceCreateFlags::SIGNALED_BIT` will be set.
    pub fn new(is_signed: bool) -> FenceCI {

        let mut fence = FenceCI { inner: FenceCI::default_ci() };

        if is_signed {
            fence.inner.flags = vk::FenceCreateFlags::SIGNALED;
        }

        fence
    }

    /// Set the `flags` member for `vk::FenceCreateInfo`.
    #[inline(always)]
    pub fn flags(mut self, flags: vk::FenceCreateFlags) {
        self.inner.flags = flags;
    }
}

impl VkObjectDiscardable for vk::Fence {

    fn discard_by(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_fence(self, None);
        }
    }
}

impl VkObjectDiscardable for &Vec<vk::Fence> {

    fn discard_by(self, device: &VkDevice) {

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
// ----------------------------------------------------------------------------------------------

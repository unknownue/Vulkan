
mod physical;
mod logical;
mod queue;

pub use self::logical::{VkLogicalDevice, VkQueue, LogicDevConfig};
pub use self::physical::{VkPhysicalDevice, PhysicalDevConfig};

use ash::vk;
use crate::utils::time::VkTimeDuration;
use crate::VkResult;
use crate::vkbytes;

pub struct VkDevice {

    pub logic : logical::VkLogicalDevice,
    pub phy   : physical::VkPhysicalDevice,
}

impl VkDevice {

    #[inline]
    pub fn build<T>(&self, ci: &T) -> VkResult<T::ObjectType>
        where
            T: crate::ci::VkObjectBuildableCI {
        ci.build(self)
    }

    #[inline]
    pub fn bind_memory(&self, object: impl VkObjectBindable, memory: vk::DeviceMemory, offset: vkbytes) -> VkResult<()> {
        object.bind(self, memory, offset)
    }

    #[inline]
    pub fn submit(&self, ci: impl VkSubmitCI, queue: vk::Queue, wait_fence: vk::Fence) -> VkResult<()> {
        ci.submit(self, queue, wait_fence)
    }

    #[inline]
    pub fn wait(&self, object: impl VkObjectWaitable, time: VkTimeDuration) -> VkResult<()> {
        object.wait(self, time)
    }

    #[inline]
    pub fn discard(&self, object: impl VkObjectCreatable) {
        object.discard(self);
    }

    #[inline]
    pub fn free<T>(&self, object: T, pool: T::AllocatePool)
        where
            T: VkObjectAllocatable {

        object.free(self, pool);
    }
}

pub trait VkObjectCreatable: Copy {

    fn discard(self, device: &VkDevice);
}

pub trait VkObjectAllocatable: Copy {
    type AllocatePool: Copy;

    fn free(self, device: &VkDevice, pool: Self::AllocatePool);
}

pub trait VkObjectBindable: Copy {

    fn bind(self, device: &VkDevice, memory: vk::DeviceMemory, offset: vkbytes) -> VkResult<()>;
}

pub trait VkObjectWaitable: Copy {

    fn wait(self, device: &VkDevice, time: VkTimeDuration) -> VkResult<()>;
}

pub trait VkSubmitCI {

    fn submit(self, device: &VkDevice, queue: vk::Queue, wait_fence: vk::Fence) -> VkResult<()>;
}

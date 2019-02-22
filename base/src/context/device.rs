
mod physical;
mod logical;
mod queue;

pub use self::logical::{VkLogicalDevice, VkQueue, LogicDevConfig};
pub use self::physical::{VkPhysicalDevice, PhysicalDevConfig};

use ash::vk;
use ash::version::DeviceV1_0;
use crate::utils::time::VkTimeDuration;
use crate::{VkResult, VkError};
use crate::{vkbytes, vkuint, vkptr};

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
    pub fn map_memory<T>(&self, memory: vk::DeviceMemory, offset: vkbytes, size: vkbytes) -> VkResult<vkptr<T>> {
        let ptr = unsafe {
            self.logic.handle.map_memory(memory, offset, size, vk::MemoryMapFlags::empty())
                .map_err(|_| VkError::device("Map Memory"))?
        };
        Ok(ptr as vkptr<T>)
    }

    #[inline]
    pub fn copy_to_ptr<T>(&self, data_ptr: vkptr, data: &[T]) {

        // implementation 1.
        unsafe {
            (data_ptr as vkptr<T>).copy_from(data.as_ptr(), data.len());
        }

        // implementation 2.
        // unsafe {
        //     let mapped_copy_target = ::std::slice::from_raw_parts_mut(data_ptr as *mut T, data.len());
        //     mapped_copy_target.copy_from_slice(data);
        // }
    }

    #[inline]
    pub fn unmap_memory(&self, memory: vk::DeviceMemory) {
        unsafe {
            self.logic.handle.unmap_memory(memory);
        }
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
    pub fn discard(&self, object: impl VkObjectDiscardable) {
        object.discard(self);
    }

    #[inline]
    pub fn free<T>(&self, object: T, pool: T::AllocatePool)
        where
            T: VkObjectAllocatable {

        object.free(self, pool);
    }

    /// Return the first memory type index that is support `request_flags`.
    #[inline]
    pub fn get_memory_type(&self, type_bits: vkuint, request_flags: vk::MemoryPropertyFlags) -> vkuint {
        use crate::utils::memory::get_memory_type_index;
        get_memory_type_index(self, type_bits, request_flags)
    }
}

pub trait VkObjectDiscardable: Copy {

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


mod physical;
mod logical;
mod queue;

pub use self::logical::{VkLogicalDevice, VkQueue, LogicDevConfig};
pub use self::physical::{VkPhysicalDevice, PhysicalDevConfig};

use ash::vk;
use ash::version::DeviceV1_0;

use crate::ci::command::{CommandPoolCI, CommandBufferAI};
use crate::ci::pipeline::PipelineCacheCI;
use crate::ci::VkObjectBuildableCI;

use crate::utils::time::VkTimeDuration;
use crate::command::{VkCmdRecorder, ITransfer};
use crate::{VkResult, VkError};
use crate::{vkbytes, vkuint, vkptr};

pub struct VkDevice {

    pub logic : VkLogicalDevice,
    pub phy   : VkPhysicalDevice,
    pub vma   : vma::Allocator,

    pub pipeline_cache: vk::PipelineCache,

    /// An internal command pool that used to allocate command buffers for data transfer operations.
    transfer_cmd_pool: vk::CommandPool,
    transfer_command : vk::CommandBuffer,
}

impl VkDevice {

    pub(super) fn new(logic: VkLogicalDevice, phy: VkPhysicalDevice, vma: vma::Allocator) -> VkResult<VkDevice> {

        let mut device = VkDevice {
            logic, phy, vma,
            pipeline_cache   : vk::PipelineCache::null(),
            transfer_cmd_pool: vk::CommandPool::null(),
            transfer_command : vk::CommandBuffer::null(),
        };

        // Create an empty pipeline cache.
        device.pipeline_cache = PipelineCacheCI::new().build(&device)?;
        // Create command pool for data data.
        device.transfer_cmd_pool = CommandPoolCI::new(device.logic.queues.transfer.family_index)
            // the command buffer allocated from this pool should short-lived and can be reset.
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER | vk::CommandPoolCreateFlags::TRANSIENT)
            .build(&device)?;
        // Create one command buffer.
        device.transfer_command = CommandBufferAI::new(device.transfer_cmd_pool, 1)
            .build(&device)?.remove(0);

        Ok(device)
    }

    pub fn get_transfer_recorder(&self) -> VkCmdRecorder<ITransfer> {

        let mut recorder = VkCmdRecorder::new(&self.logic, self.transfer_command);
        recorder.set_usage(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        recorder
    }

    pub fn flush_transfer(&self, recorder: VkCmdRecorder<ITransfer>) -> VkResult<()> {

        recorder.flush_copy_command(self.logic.queues.transfer.handle)?;

        // reset the command buffer after transfer operation has been done.
        unsafe {
            self.logic.handle.reset_command_buffer(self.transfer_command, vk::CommandBufferResetFlags::RELEASE_RESOURCES)
                .map_err(|_| VkError::device("Reset Command Buffer"))
        }
    }

    pub(super) fn drop_self(self) {

        self.discard(self.transfer_cmd_pool);
        self.discard(self.pipeline_cache);
        // destroy vma manually, so that vma will be destroyed before logic device.
        drop(self.vma);
    }
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
        object.discard_by(self);
    }

    #[inline]
    pub fn vma_discard(&mut self, object: impl VmaResourceDiscardable) -> VkResult<()> {
        object.discard_by(&mut self.vma)
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

    fn discard_by(self, device: &VkDevice);
}

pub trait VmaResourceDiscardable {

    fn discard_by(self, vma: &mut vma::Allocator) -> VkResult<()>;
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


use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::{VkDevice, VkObjectDiscardable, VkObjectBindable};
use crate::ci::{VulkanCI, VkObjectBuildableCI};
use crate::error::{VkResult, VkError};
use crate::{vkuint, vkbytes};

use std::ptr;

// ----------------------------------------------------------------------------------------------
/// Wrapper class for `vk::BufferCreateInfo`.
///
/// The default values are defined as follows:
/// ``` ignore
/// vk::BufferCreateInfo {
///     s_type: vk::StructureType::BUFFER_CREATE_INFO,
///     p_next: ptr::null(),
///     flags : vk::BufferCreateFlags::empty(),
///     size  : 0,
///     usage : vk::BufferUsageFlags::empty(),
///     sharing_mode: vk::SharingMode::EXCLUSIVE,
///     queue_family_index_count: 0,
///     p_queue_family_indices  : ptr::null(),
/// }
/// ```
///
/// See [VkBufferCreateInfo](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkBufferCreateInfo.html) for more detail.
///
#[derive(Debug, Clone)]
pub struct BufferCI {
    inner: vk::BufferCreateInfo,
    queue_families: Option<Vec<vkuint>>,
}

impl VulkanCI<vk::BufferCreateInfo> for BufferCI {

    fn default_ci() -> vk::BufferCreateInfo {

        vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            p_next: ptr::null(),
            flags : vk::BufferCreateFlags::empty(),
            size  : 0,
            usage : vk::BufferUsageFlags::empty(),
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices  : ptr::null(),
        }
    }
}

impl AsRef<vk::BufferCreateInfo> for BufferCI {

    fn as_ref(&self) -> &vk::BufferCreateInfo {
        &self.inner
    }
}

impl VkObjectBuildableCI for BufferCI {
    type ObjectType = (vk::Buffer, vk::MemoryRequirements);

    /// Create `vk::Buffer` object, and return its handle and memory requirement.
    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        debug_assert_ne!(self.inner.usage, vk::BufferUsageFlags::empty(), "the usage member of vk::BufferCreateInfo must not be 0!");

        let buffer = unsafe {
            device.logic.handle.create_buffer(self.as_ref(), None)
                .map_err(|_| VkError::create("Buffer"))?
        };

        let requirement = unsafe {
            device.logic.handle.get_buffer_memory_requirements(buffer)
        };

        Ok((buffer, requirement))
    }
}

impl BufferCI {

    /// Initialize `vk::BufferCreateInfo` with default value.
    ///
    /// `size` is the size in bytes of buffer.
    pub fn new(size: vkbytes) -> BufferCI {

        debug_assert!(size > 0, "size must be greater than 0!");

        BufferCI {
            inner: vk::BufferCreateInfo {
                size,
                ..BufferCI::default_ci()
            },
            queue_families: None,
        }
    }

    /// Set the `flags` member for `vk::BufferCreateInfo`.
    ///
    /// It describes additional parameters of the buffer.
    #[inline(always)]
    pub fn flags(mut self, flags: vk::BufferCreateFlags) -> BufferCI {
        self.inner.flags = flags; self
    }

    /// Set the `usage` member for `vk::BufferCreateInfo`.
    ///
    /// It specifies allowed usages of the buffer. The member must be set before creating `vk::Buffer` object.
    #[inline(always)]
    pub fn usage(mut self, flags: vk::BufferUsageFlags) -> BufferCI {
        self.inner.usage = flags; self
    }

    /// Set the list of queue families that will access this buffer.
    ///
    /// The `sharing_mode` member of `vk::BufferCreateInfo` will be set to `vk::SharingMode::CONCURRENT` automatically.
    #[inline(always)]
    pub fn sharing_queues(mut self, families_indices: Vec<vkuint>) -> BufferCI {

        self.inner.queue_family_index_count = families_indices.len() as _;
        self.inner.p_queue_family_indices   = families_indices.as_ptr();

        debug_assert!(self.inner.queue_family_index_count > 1, "The number of shared queue families must be greater than 1!");

        self.queue_families = Some(families_indices);
        self.inner.sharing_mode = vk::SharingMode::CONCURRENT; self
    }
}

impl VkObjectDiscardable for vk::Buffer {

    fn discard_by(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_buffer(self, None)
        }
    }
}

impl VkObjectBindable for vk::Buffer {

    /// Bind a specific range of `memory` to this buffer.
    fn bind(self, device: &VkDevice, memory: vk::DeviceMemory, offset: vkbytes) -> VkResult<()> {
        unsafe {
            device.logic.handle.bind_buffer_memory(self, memory, offset)
                .map_err(|_| VkError::device("Binding Buffer Memory"))
        }
    }
}
// ----------------------------------------------------------------------------------------------

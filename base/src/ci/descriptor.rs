//! Types which simplify the creation of Vulkan descriptor objects.

use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::VkDevice;
use crate::context::{VkObjectDiscardable, VkObjectAllocatable};
use crate::ci::{VulkanCI, VkObjectBuildableCI};
use crate::error::{VkResult, VkError};
use crate::vkuint;

use std::ptr;

// ----------------------------------------------------------------------------------------------
/// Wrapper class for `vk::DescriptorPoolCreateInfo`.
///
/// The default values are defined as follows:
/// ``` ignore
/// vk::DescriptorPoolCreateInfo {
///     s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
///     p_next: ptr::null(),
///     flags : vk::DescriptorPoolCreateFlags::empty(),
///     max_sets: 0,
///     pool_size_count: 0,
///     p_pool_sizes   : ptr::null(),
/// }
/// ```
///
/// See [VkDescriptorPoolCreateInfo](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkDescriptorPoolCreateInfo.html) for more detail.
///
#[derive(Debug, Clone)]
pub struct DescriptorPoolCI {

    inner: vk::DescriptorPoolCreateInfo,
    pool_sizes: Vec<vk::DescriptorPoolSize>,
}

impl VulkanCI<vk::DescriptorPoolCreateInfo> for DescriptorPoolCI {

    fn default_ci() -> vk::DescriptorPoolCreateInfo {

        vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags : vk::DescriptorPoolCreateFlags::empty(),
            max_sets: 0,
            pool_size_count: 0,
            p_pool_sizes   : ptr::null(),
        }
    }
}

impl AsRef<vk::DescriptorPoolCreateInfo> for DescriptorPoolCI {

    fn as_ref(&self) -> &vk::DescriptorPoolCreateInfo {
        &self.inner
    }
}

impl VkObjectBuildableCI for DescriptorPoolCI {
    type ObjectType = vk::DescriptorPool;

    /// Create `vk::DescriptorPool` object, and return its handle.
    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        debug_assert!(!self.pool_sizes.is_empty(), "The count of pool sizes must be greater than 0!");

        let descriptor_pool = unsafe {
            device.logic.handle.create_descriptor_pool(self.as_ref(), None)
                .map_err(|_| VkError::create("Descriptor Pool"))?
        };
        Ok(descriptor_pool)
    }
}

impl DescriptorPoolCI {

    /// Initialize `vk::DescriptorPoolCreateInfo` with default value.
    ///
    /// `max_set_count` is the maximum number of descriptor sets that this descriptor pool may allocated.
    pub fn new(max_set_count: vkuint) -> DescriptorPoolCI {

        debug_assert!(max_set_count > 0, "max_set_count must be greater than 0!");

        DescriptorPoolCI {
            inner: vk::DescriptorPoolCreateInfo {
                max_sets: max_set_count,
                ..DescriptorPoolCI::default_ci()
            },
            pool_sizes: Vec::new(),
        }
    }

    /// Set the `flags` member for `vk::DescriptorPoolCreateInfo`.
    ///
    /// It specifies the some supported operations for this pool.
    #[inline(always)]
    pub fn flags(mut self, flags: vk::DescriptorPoolCreateFlags) -> DescriptorPoolCI {
        self.inner.flags = flags; self
    }

    /// Add a new descriptor type that can be allocated by this pool.
    ///
    /// `type_` is the type of descriptor.
    ///
    /// `count` is tha maximum number of this descriptor that can be allocated by this pool.
    #[inline]
    pub fn add_descriptor(mut self, type_: vk::DescriptorType, count: vkuint) -> DescriptorPoolCI {

        debug_assert!(count > 0, "The count of descriptor must be greater than 0!");

        self.pool_sizes.push(vk::DescriptorPoolSize {
            ty: type_,
            descriptor_count: count,
        });
        self.inner.pool_size_count = self.pool_sizes.len() as _;
        self.inner.p_pool_sizes    = self.pool_sizes.as_ptr(); self
    }
}

impl VkObjectDiscardable for vk::DescriptorPool {

    fn discard_by(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_descriptor_pool(self, None);
        }
    }
}
// ----------------------------------------------------------------------------------------------


// ----------------------------------------------------------------------------------------------
/// Wrapper class for `vk::DescriptorSetLayoutCreateInfo`.
///
/// The default values are defined as follows:
/// ``` ignore
/// vk::DescriptorSetLayoutCreateInfo {
///     s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
///     p_next: ptr::null(),
///     flags: vk::DescriptorSetLayoutCreateFlags::empty(),
///     binding_count: 0,
///     p_bindings   : ptr::null(),
/// }
/// ```
///
/// See [VkDescriptorSetLayoutCreateInfo](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkDescriptorSetLayoutCreateInfo.html) for more detail.
///
#[derive(Debug, Clone)]
pub struct DescriptorSetLayoutCI {
    inner: vk::DescriptorSetLayoutCreateInfo,
    bindings: Vec<vk::DescriptorSetLayoutBinding>,
}

impl VulkanCI<vk::DescriptorSetLayoutCreateInfo> for DescriptorSetLayoutCI {

    fn default_ci() -> vk::DescriptorSetLayoutCreateInfo {

        vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DescriptorSetLayoutCreateFlags::empty(),
            binding_count: 0,
            p_bindings   : ptr::null(),
        }
    }
}

impl AsRef<vk::DescriptorSetLayoutCreateInfo> for DescriptorSetLayoutCI {

    fn as_ref(&self) -> &vk::DescriptorSetLayoutCreateInfo {
        &self.inner
    }
}

impl VkObjectBuildableCI for DescriptorSetLayoutCI {
    type ObjectType = vk::DescriptorSetLayout;

    /// Create `vk::DescriptorSetLayout` object, and return its handle.
    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let descriptor_set_layout = unsafe {
            device.logic.handle.create_descriptor_set_layout(self.as_ref(), None)
                .map_err(|_| VkError::create("Descriptor Set Layout"))?
        };
        Ok(descriptor_set_layout)
    }
}

impl DescriptorSetLayoutCI {

    /// Initialize `vk::DescriptorSetLayoutCreateInfo` with default value.
    #[inline(always)]
    pub fn new() -> DescriptorSetLayoutCI {

        DescriptorSetLayoutCI {
            inner: DescriptorSetLayoutCI::default_ci(),
            bindings: Vec::new(),
        }
    }

    /// Add set layout bindings to this descriptor set.
    #[inline(always)]
    pub fn add_binding(mut self, binding: vk::DescriptorSetLayoutBinding) -> DescriptorSetLayoutCI {

        self.bindings.push(binding);
        self.inner.binding_count = self.bindings.len() as _;
        self.inner.p_bindings    = self.bindings.as_ptr(); self
    }

    /// Set the `flags` member for `vk::DescriptorSetLayoutCreateInfo`.
    ///
    /// It specifies options for descriptor set layout creation.
    #[inline(always)]
    pub fn flags(mut self, flags: vk::DescriptorSetLayoutCreateFlags) -> DescriptorSetLayoutCI {
        self.inner.flags = flags; self
    }
}

impl VkObjectDiscardable for vk::DescriptorSetLayout {

    fn discard_by(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_descriptor_set_layout(self, None);
        }
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
/// Wrapper class for `vk::DescriptorSetAllocateInfo`.
///
/// The default values are defined as follows:
/// ``` ignore
/// vk::DescriptorSetAllocateInfo {
///     s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
///     p_next: ptr::null(),
///     descriptor_pool: vk::DescriptorPool::null(),
///     descriptor_set_count: 0,
///     p_set_layouts       : ptr::null(),
/// }
/// ```
///
/// See [VkDescriptorSetAllocateInfo](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkDescriptorSetAllocateInfo.html) for more detail.
///
#[derive(Debug, Clone)]
pub struct DescriptorSetAI {
    inner: vk::DescriptorSetAllocateInfo,
    set_layouts: Vec<vk::DescriptorSetLayout>,
}

impl VulkanCI<vk::DescriptorSetAllocateInfo> for DescriptorSetAI {

    fn default_ci() -> vk::DescriptorSetAllocateInfo {

        vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            p_next: ptr::null(),
            descriptor_pool: vk::DescriptorPool::null(),
            descriptor_set_count: 0,
            p_set_layouts       : ptr::null(),
        }
    }
}

impl AsRef<vk::DescriptorSetAllocateInfo> for DescriptorSetAI {

    fn as_ref(&self) -> &vk::DescriptorSetAllocateInfo {
        &self.inner
    }
}

impl VkObjectBuildableCI for DescriptorSetAI {
    type ObjectType = Vec<vk::DescriptorSet>;

    /// Create `vk::DescriptorSet` objects, and return their handles.
    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        debug_assert!(!self.set_layouts.is_empty(), "Descriptor sets count must be greater than 0!");

        let descriptor_sets = unsafe {
            device.logic.handle.allocate_descriptor_sets(self.as_ref())
                .map_err(|_| VkError::create("Allocate Descriptor Set"))?
        };
        Ok(descriptor_sets)
    }
}

impl DescriptorSetAI {

    /// Initialize `vk::DescriptorSetAllocateInfo` with default value.
    ///
    /// `pool` is the pool where these sets will be allocated from.
    pub fn new(pool: vk::DescriptorPool) -> DescriptorSetAI {

        DescriptorSetAI {
            inner: vk::DescriptorSetAllocateInfo {
                descriptor_pool: pool,
                ..DescriptorSetAI::default_ci()
            },
            set_layouts: Vec::new(),
        }
    }

    /// Add a new descriptor set layout to this allocations.
    #[inline(always)]
    pub fn add_set_layout(mut self, set_layout: vk::DescriptorSetLayout) -> DescriptorSetAI {

        self.set_layouts.push(set_layout);
        self.inner.descriptor_set_count = self.set_layouts.len() as _;
        self.inner.p_set_layouts        = self.set_layouts.as_ptr(); self
    }
}

impl VkObjectAllocatable for vk::DescriptorSet {
    type AllocatePool = vk::DescriptorPool;

    fn free(self, device: &VkDevice, pool: Self::AllocatePool) {
        unsafe {
            device.logic.handle.free_descriptor_sets(pool, &[self])
        }
    }
}

impl VkObjectAllocatable for &[vk::DescriptorSet] {
    type AllocatePool = vk::DescriptorPool;

    fn free(self, device: &VkDevice, pool: Self::AllocatePool) {
        unsafe {
            device.logic.handle.free_descriptor_sets(pool, self)
        }
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
/// Wrapper class for `vk::WriteDescriptorSet`(for `vk::Buffer`).
///
/// The default values are defined as follows:
/// ``` ignore
/// vk::WriteDescriptorSet {
///     s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
///     p_next: ptr::null(),
///     dst_set: vk::DescriptorSet::null(),
///     dst_binding: 0,
///     dst_array_element   : 0,
///     descriptor_count    : 0,
///     descriptor_type     : vk::DescriptorType::UNIFORM_BUFFER,
///     p_image_info        : ptr::null(),
///     p_buffer_info       : ptr::null(),
///     p_texel_buffer_view : ptr::null(),
/// }
/// ```
///
/// See [VkWriteDescriptorSet](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkWriteDescriptorSet.html) for more detail.
///
#[derive(Debug, Clone)]
pub struct DescriptorBufferSetWI {

    inner: vk::WriteDescriptorSet,
    writes: Vec<vk::DescriptorBufferInfo>,
}

impl VulkanCI<vk::WriteDescriptorSet> for DescriptorBufferSetWI {

    fn default_ci() -> vk::WriteDescriptorSet {

        vk::WriteDescriptorSet {
            s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
            p_next: ptr::null(),
            dst_set: vk::DescriptorSet::null(),
            dst_binding: 0,
            dst_array_element   : 0,
            descriptor_count    : 0,
            descriptor_type     : vk::DescriptorType::UNIFORM_BUFFER,
            p_image_info        : ptr::null(),
            p_buffer_info       : ptr::null(),
            p_texel_buffer_view : ptr::null(),
        }
    }
}

impl AsRef<vk::WriteDescriptorSet> for DescriptorBufferSetWI {

    fn as_ref(&self) -> &vk::WriteDescriptorSet {
        &self.inner
    }
}

impl DescriptorBufferSetWI {

    /// Initialize `vk::WriteDescriptorSet` with default value.
    ///
    /// `set` is the destination descriptor set to update.
    ///
    /// `bindings` is the descriptor binding within the set.
    ///
    /// `type_` specifies the type of buffer descriptor to update.
    pub fn new(set: vk::DescriptorSet, binding: vkuint, type_: vk::DescriptorType) -> DescriptorBufferSetWI {

        DescriptorBufferSetWI {
            inner: vk::WriteDescriptorSet {
                dst_set: set,
                dst_binding: binding,
                descriptor_type: type_,
                ..DescriptorBufferSetWI::default_ci()
            },
            writes: Vec::new(),
        }
    }

    /// Add a new buffer descriptor to update for the set.
    #[inline(always)]
    pub fn add_buffer(mut self, info: vk::DescriptorBufferInfo) -> DescriptorBufferSetWI {

        self.writes.push(info);
        self.inner.descriptor_count = self.writes.len() as _;
        self.inner.p_buffer_info    = self.writes.as_ptr(); self
    }

    /// Reset all buffer descriptors to update for the set.
    #[inline(always)]
    pub fn set_buffer(&mut self, infos: Vec<vk::DescriptorBufferInfo>) {

        self.inner.descriptor_count = infos.len() as _;
        self.inner.p_buffer_info = infos.as_ptr();

        self.writes = infos;
    }

    /// Set the `array_element` member for `vk::WriteDescriptorSet`.
    ///
    /// It is the starting element index in the descriptor array.
    #[inline(always)]
    pub fn dst_array_element(mut self, array_element: vkuint) -> DescriptorBufferSetWI {
        self.inner.dst_array_element = array_element; self
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
/// Wrapper class for `vk::WriteDescriptorSet`(for `vk::Image`).
///
/// The default values are defined as follows:
/// ``` ignore
/// vk::WriteDescriptorSet {
///     s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
///     p_next: ptr::null(),
///     dst_set: vk::DescriptorSet::null(),
///     dst_binding: 0,
///     dst_array_element   : 0,
///     descriptor_count    : 0,
///     descriptor_type     : vk::DescriptorType::UNIFORM_BUFFER,
///     p_image_info        : ptr::null(),
///     p_buffer_info       : ptr::null(),
///     p_texel_buffer_view : ptr::null(),
/// }
/// ```
///
/// See [VkWriteDescriptorSet](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkWriteDescriptorSet.html) for more detail.
///
#[derive(Debug, Clone)]
pub struct DescriptorImageSetWI {

    inner: vk::WriteDescriptorSet,
    writes: Vec<vk::DescriptorImageInfo>,
}

impl VulkanCI<vk::WriteDescriptorSet> for DescriptorImageSetWI {

    fn default_ci() -> vk::WriteDescriptorSet{

        vk::WriteDescriptorSet {
            s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
            p_next: ptr::null(),
            dst_set: vk::DescriptorSet::null(),
            dst_binding: 0,
            dst_array_element   : 0,
            descriptor_count    : 0,
            descriptor_type     : vk::DescriptorType::UNIFORM_BUFFER,
            p_image_info        : ptr::null(),
            p_buffer_info       : ptr::null(),
            p_texel_buffer_view : ptr::null(),
        }
    }
}

impl AsRef<vk::WriteDescriptorSet> for DescriptorImageSetWI {

    fn as_ref(&self) -> &vk::WriteDescriptorSet {
        &self.inner
    }
}

impl DescriptorImageSetWI {

    /// Initialize `vk::WriteDescriptorSet` with default value.
    ///
    /// `set` is the destination descriptor set to update.
    ///
    /// `bindings` is the descriptor binding within the set.
    ///
    /// `type_` specifies the type of image descriptor to update.
    pub fn new(set: vk::DescriptorSet, binding: vkuint, type_: vk::DescriptorType) -> DescriptorImageSetWI {

        DescriptorImageSetWI {
            inner: vk::WriteDescriptorSet {
                dst_set: set,
                dst_binding: binding,
                descriptor_type: type_,
                ..DescriptorImageSetWI::default_ci()
            },
            writes: Vec::new(),
        }
    }

    /// Add a new image descriptor to update for the set.
    #[inline(always)]
    pub fn add_image(mut self, info: vk::DescriptorImageInfo) -> DescriptorImageSetWI {

        self.writes.push(info);
        self.inner.descriptor_count = self.writes.len() as _;
        self.inner.p_image_info     = self.writes.as_ptr(); self
    }

    /// Reset all image descriptors to update for the set.
    #[inline(always)]
    pub fn set_images(&mut self, infos: Vec<vk::DescriptorImageInfo>) {

        self.inner.descriptor_count = infos.len() as _;
        self.inner.p_image_info     = infos.as_ptr();

        self.writes = infos;
    }

    /// Set the `array_element` member for `vk::WriteDescriptorSet`.
    ///
    /// It is the starting element index in the descriptor array.
    #[inline(always)]
    pub fn dst_array_element(mut self, array_element: vkuint) -> DescriptorImageSetWI {
        self.inner.dst_array_element = array_element; self
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
/// Utility type to update descriptor set.
#[derive(Default)]
pub struct DescriptorSetsUpdateCI<'a> {

    writes: Vec<vk::WriteDescriptorSet>,
    copies: Vec<vk::CopyDescriptorSet>,

    phantom_type: ::std::marker::PhantomData<& 'a ()>,
}

pub trait DescriptorSetWritable: AsRef<vk::WriteDescriptorSet> {}

impl DescriptorSetWritable for DescriptorBufferSetWI {}
impl DescriptorSetWritable for DescriptorImageSetWI  {}

impl<'a, 'b: 'a> DescriptorSetsUpdateCI<'a> {

    /// Constructor of `DescriptorSetsUpdateCI`.
    #[inline(always)]
    pub fn new() -> DescriptorSetsUpdateCI<'a> {
        DescriptorSetsUpdateCI::default()
    }

    /// Add a `DescriptorBufferSetWI` or `DescriptorImageSetWI` to the descriptor update sequences.
    #[inline(always)]
    pub fn add_write(mut self, value: &'b impl DescriptorSetWritable) -> DescriptorSetsUpdateCI<'a> {
        self.writes.push(value.as_ref().clone()); self
    }

    #[inline(always)]
    pub fn add_copy(mut self, value: &'b vk::CopyDescriptorSet) -> DescriptorSetsUpdateCI<'a> {
        self.copies.push(value.clone()); self
    }

    /// Execute the descriptor sets update operations.
    #[inline(always)]
    pub fn update(self, device: &VkDevice) {

        unsafe {
            device.logic.handle.update_descriptor_sets(&self.writes, &self.copies);
        }
    }
}
// ----------------------------------------------------------------------------------------------

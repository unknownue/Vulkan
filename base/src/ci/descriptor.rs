
use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::VkDevice;
use crate::context::{VkObjectDiscardable, VkObjectAllocatable};
use crate::ci::{VulkanCI, VkObjectBuildableCI};
use crate::error::{VkResult, VkError};
use crate::vkuint;

use std::ptr;
use std::ops::Deref;


// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::DescriptorPoolCreateInfo.
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

impl Deref for DescriptorPoolCI {
    type Target = vk::DescriptorPoolCreateInfo;

    fn deref(&self) -> &vk::DescriptorPoolCreateInfo {
        &self.inner
    }
}

impl VkObjectBuildableCI for DescriptorPoolCI {
    type ObjectType = vk::DescriptorPool;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let descriptor_pool = unsafe {
            device.logic.handle.create_descriptor_pool(self, None)
                .map_err(|_| VkError::create("Descriptor Pool"))?
        };
        Ok(descriptor_pool)
    }
}

impl DescriptorPoolCI {

    pub fn new(max_set_count: vkuint) -> DescriptorPoolCI {

        DescriptorPoolCI {
            inner: vk::DescriptorPoolCreateInfo {
                max_sets: max_set_count,
                ..DescriptorPoolCI::default_ci()
            },
            pool_sizes: Vec::new(),
        }
    }

    #[inline(always)]
    pub fn flags(mut self, flags: vk::DescriptorPoolCreateFlags) -> DescriptorPoolCI {
        self.inner.flags = flags; self
    }

    #[inline]
    pub fn add_descriptor(mut self, r#type: vk::DescriptorType, count: vkuint) -> DescriptorPoolCI {

        self.pool_sizes.push(vk::DescriptorPoolSize {
            ty: r#type,
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
/// Wrapper class for vk::DescriptorSetLayoutCreateInfo.
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

impl Deref for DescriptorSetLayoutCI {
    type Target = vk::DescriptorSetLayoutCreateInfo;

    fn deref(&self) -> &vk::DescriptorSetLayoutCreateInfo {
        &self.inner
    }
}

impl VkObjectBuildableCI for DescriptorSetLayoutCI {
    type ObjectType = vk::DescriptorSetLayout;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let descriptor_set_layout = unsafe {
            device.logic.handle.create_descriptor_set_layout(self, None)
                .map_err(|_| VkError::create("Descriptor Set Layout"))?
        };
        Ok(descriptor_set_layout)
    }
}

impl DescriptorSetLayoutCI {

    pub fn new() -> DescriptorSetLayoutCI {

        DescriptorSetLayoutCI {
            inner: DescriptorSetLayoutCI::default_ci(),
            bindings: Vec::new(),
        }
    }

    #[inline(always)]
    pub fn add_binding(mut self, binding: vk::DescriptorSetLayoutBinding) -> DescriptorSetLayoutCI {

        self.bindings.push(binding);
        self.inner.binding_count = self.bindings.len() as _;
        self.inner.p_bindings    = self.bindings.as_ptr(); self
    }

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
/// Wrapper class for vk::DescriptorSetAllocateInfo.
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

impl Deref for DescriptorSetAI {
    type Target = vk::DescriptorSetAllocateInfo;

    fn deref(&self) -> &vk::DescriptorSetAllocateInfo {
        &self.inner
    }
}

impl VkObjectBuildableCI for DescriptorSetAI {
    type ObjectType = Vec<vk::DescriptorSet>;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let descriptor_sets = unsafe {
            device.logic.handle.allocate_descriptor_sets(self)
                .map_err(|_| VkError::create("Allocate Descriptor Set"))?
        };
        Ok(descriptor_sets)
    }
}

impl DescriptorSetAI {

    pub fn new(pool: vk::DescriptorPool) -> DescriptorSetAI {

        DescriptorSetAI {
            inner: vk::DescriptorSetAllocateInfo {
                descriptor_pool: pool,
                ..DescriptorSetAI::default_ci()
            },
            set_layouts: Vec::new(),
        }
    }

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
/// Wrapper class for vk::WriteDescriptorSet(for vk::Buffer).
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

impl Deref for DescriptorBufferSetWI {
    type Target = vk::WriteDescriptorSet;

    fn deref(&self) -> &vk::WriteDescriptorSet {
        &self.inner
    }
}

impl DescriptorBufferSetWI {

    pub fn new(set: vk::DescriptorSet, binding: vkuint, r#type: vk::DescriptorType) -> DescriptorBufferSetWI {

        DescriptorBufferSetWI {
            inner: vk::WriteDescriptorSet {
                dst_set: set,
                dst_binding: binding,
                descriptor_type: r#type,
                ..DescriptorBufferSetWI::default_ci()
            },
            writes: Vec::new(),
        }
    }

    #[inline(always)]
    pub fn add_buffer(mut self, info: vk::DescriptorBufferInfo) -> DescriptorBufferSetWI {

        self.writes.push(info);
        self.inner.descriptor_count = self.writes.len() as _;
        self.inner.p_buffer_info    = self.writes.as_ptr(); self
    }

    #[inline(always)]
    pub fn set_buffer(&mut self, infos: Vec<vk::DescriptorBufferInfo>) {

        self.inner.descriptor_count = infos.len() as _;
        self.inner.p_buffer_info = infos.as_ptr();

        self.writes = infos;
    }

    #[inline(always)]
    pub fn dst_array_element(mut self, array_element: vkuint) -> DescriptorBufferSetWI {
        self.inner.dst_array_element = array_element; self
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::WriteDescriptorSet(for vk::Image).
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

impl Deref for DescriptorImageSetWI {
    type Target = vk::WriteDescriptorSet;

    fn deref(&self) -> &vk::WriteDescriptorSet {
        &self.inner
    }
}

impl DescriptorImageSetWI {

    pub fn new(set: vk::DescriptorSet, binding: vkuint, r#type: vk::DescriptorType) -> DescriptorImageSetWI {

        DescriptorImageSetWI {
            inner: vk::WriteDescriptorSet {
                dst_set: set,
                dst_binding: binding,
                descriptor_type: r#type,
                ..DescriptorImageSetWI::default_ci()
            },
            writes: Vec::new(),
        }
    }

    #[inline(always)]
    pub fn add_image(mut self, info: vk::DescriptorImageInfo) -> DescriptorImageSetWI {

        self.writes.push(info);
        self.inner.descriptor_count = self.writes.len() as _;
        self.inner.p_image_info     = self.writes.as_ptr(); self
    }

    #[inline(always)]
    pub fn set_images(&mut self, infos: Vec<vk::DescriptorImageInfo>) {

        self.inner.descriptor_count = infos.len() as _;
        self.inner.p_image_info     = infos.as_ptr();

        self.writes = infos;
    }

    #[inline(always)]
    pub fn dst_array_element(mut self, array_element: vkuint) -> DescriptorImageSetWI {
        self.inner.dst_array_element = array_element; self
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
/// Convenient struct to update descriptor set.
#[derive(Default)]
pub struct DescriptorSetsUpdateCI<'a> {

    writes: Vec<vk::WriteDescriptorSet>,
    copies: Vec<vk::CopyDescriptorSet>,

    phantom_type: ::std::marker::PhantomData<& 'a ()>,
}

pub trait DescriptorSetWritable: Deref<Target=vk::WriteDescriptorSet> {}

impl DescriptorSetWritable for DescriptorBufferSetWI {}
impl DescriptorSetWritable for DescriptorImageSetWI  {}

impl<'a, 'b: 'a> DescriptorSetsUpdateCI<'a> {

    #[inline(always)]
    pub fn new() -> DescriptorSetsUpdateCI<'a> {
        DescriptorSetsUpdateCI::default()
    }

    #[inline(always)]
    pub fn add_write(mut self, value: &'b impl DescriptorSetWritable) -> DescriptorSetsUpdateCI<'a> {
        self.writes.push(value.deref().clone()); self
    }

    #[inline(always)]
    pub fn add_copy(mut self, value: &'b vk::CopyDescriptorSet) -> DescriptorSetsUpdateCI<'a> {
        self.copies.push(value.clone()); self
    }

    #[inline(always)]
    pub fn update(self, device: &VkDevice) {

        unsafe {
            device.logic.handle.update_descriptor_sets(&self.writes, &self.copies);
        }
    }
}
// ----------------------------------------------------------------------------------------------

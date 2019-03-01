
use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::VkDevice;
use crate::context::{VkObjectDiscardable, VkObjectAllocatable};
use crate::ci::{VulkanCI, VkObjectBuildableCI};
use crate::error::{VkResult, VkError};
use crate::vkuint;

use std::ptr;

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::DescriptorPoolCreateInfo.
#[derive(Debug, Clone)]
pub struct DescriptorPoolCI {

    ci: vk::DescriptorPoolCreateInfo,
    pool_sizes: Vec<vk::DescriptorPoolSize>,
}

impl VulkanCI for DescriptorPoolCI {
    type CIType = vk::DescriptorPoolCreateInfo;

    fn default_ci() -> Self::CIType {

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

impl VkObjectBuildableCI for DescriptorPoolCI {
    type ObjectType = vk::DescriptorPool;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let pool_ci = vk::DescriptorPoolCreateInfo {
            pool_size_count: self.pool_sizes.len() as _,
            p_pool_sizes   : self.pool_sizes.as_ptr(),
            ..self.ci
        };

        let descriptor_pool = unsafe {
            device.logic.handle.create_descriptor_pool(&pool_ci, None)
                .map_err(|_| VkError::create("Descriptor Pool"))?
        };
        Ok(descriptor_pool)
    }
}

impl DescriptorPoolCI {

    pub fn new(max_set_count: vkuint) -> DescriptorPoolCI {

        DescriptorPoolCI {
            ci: vk::DescriptorPoolCreateInfo {
                max_sets: max_set_count,
                ..DescriptorPoolCI::default_ci()
            },
            pool_sizes: Vec::new(),
        }
    }

    #[inline(always)]
    pub fn flags(mut self, flags: vk::DescriptorPoolCreateFlags) -> DescriptorPoolCI {
        self.ci.flags = flags; self
    }

    #[inline(always)]
    pub fn add_descriptor(mut self, r#type: vk::DescriptorType, count: vkuint) -> DescriptorPoolCI {
        self.pool_sizes.push(vk::DescriptorPoolSize {
            ty: r#type,
            descriptor_count: count,
        }); self
    }
}

impl VkObjectDiscardable for vk::DescriptorPool {

    fn discard(self, device: &VkDevice) {
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
    ci: vk::DescriptorSetLayoutCreateInfo,
    bindings: Vec<vk::DescriptorSetLayoutBinding>,
}

impl VulkanCI for DescriptorSetLayoutCI {
    type CIType = vk::DescriptorSetLayoutCreateInfo;

    fn default_ci() -> Self::CIType {

        vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DescriptorSetLayoutCreateFlags::empty(),
            binding_count: 0,
            p_bindings   : ptr::null(),
        }
    }
}

impl VkObjectBuildableCI for DescriptorSetLayoutCI {
    type ObjectType = vk::DescriptorSetLayout;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let set_layout_ci = vk::DescriptorSetLayoutCreateInfo {
            binding_count: self.bindings.len() as _,
            p_bindings   : self.bindings.as_ptr(),
            ..self.ci
        };

        let descriptor_set_layout = unsafe {
            device.logic.handle.create_descriptor_set_layout(&set_layout_ci, None)
                .map_err(|_| VkError::create("Descriptor Set Layout"))?
        };
        Ok(descriptor_set_layout)
    }
}

impl DescriptorSetLayoutCI {

    pub fn new() -> DescriptorSetLayoutCI {

        DescriptorSetLayoutCI {
            ci: DescriptorSetLayoutCI::default_ci(),
            bindings: Vec::new(),
        }
    }

    #[inline(always)]
    pub fn add_binding(mut self, binding: vk::DescriptorSetLayoutBinding) -> DescriptorSetLayoutCI {
        self.bindings.push(binding); self
    }

    #[inline(always)]
    pub fn flags(mut self, flags: vk::DescriptorSetLayoutCreateFlags) -> DescriptorSetLayoutCI {
        self.ci.flags = flags; self
    }
}

impl VkObjectDiscardable for vk::DescriptorSetLayout {

    fn discard(self, device: &VkDevice) {
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
    ai: vk::DescriptorSetAllocateInfo,
    set_layouts: Vec<vk::DescriptorSetLayout>,
}

impl VulkanCI for DescriptorSetAI {
    type CIType = vk::DescriptorSetAllocateInfo;

    fn default_ci() -> Self::CIType {

        vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            p_next: ptr::null(),
            descriptor_pool: vk::DescriptorPool::null(),
            descriptor_set_count: 0,
            p_set_layouts       : ptr::null(),
        }
    }
}

impl VkObjectBuildableCI for DescriptorSetAI {
    type ObjectType = Vec<vk::DescriptorSet>;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let sets_ai = vk::DescriptorSetAllocateInfo {
            descriptor_set_count: self.set_layouts.len() as _,
            p_set_layouts       : self.set_layouts.as_ptr(),
            ..self.ai
        };

        let descriptor_sets = unsafe {
            device.logic.handle.allocate_descriptor_sets(&sets_ai)
                .map_err(|_| VkError::create("Allocate Descriptor Set"))?
        };
        Ok(descriptor_sets)
    }
}

impl DescriptorSetAI {

    pub fn new(pool: vk::DescriptorPool) -> DescriptorSetAI {

        DescriptorSetAI {
            ai: vk::DescriptorSetAllocateInfo {
                descriptor_pool: pool,
                ..DescriptorSetAI::default_ci()
            },
            set_layouts: Vec::new(),
        }
    }

    pub fn add_set_layout(mut self, set_layout: vk::DescriptorSetLayout) -> DescriptorSetAI {
        self.set_layouts.push(set_layout); self
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
    wi: vk::WriteDescriptorSet,
    writes: Vec<vk::DescriptorBufferInfo>,
}

impl VulkanCI for DescriptorBufferSetWI {
    type CIType = vk::WriteDescriptorSet;

    fn default_ci() -> Self::CIType {

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

impl DescriptorBufferSetWI {

    pub fn new(set: vk::DescriptorSet, binding: vkuint, r#type: vk::DescriptorType) -> DescriptorBufferSetWI {

        DescriptorBufferSetWI {
            wi: vk::WriteDescriptorSet {
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
        self.writes.push(info); self
    }

    #[inline(always)]
    pub fn set_buffer(&mut self, infos: Vec<vk::DescriptorBufferInfo>) {
        self.writes = infos;
    }

    #[inline(always)]
    pub fn dst_array_element(mut self, array_element: vkuint) -> DescriptorBufferSetWI {
        self.wi.dst_array_element = array_element; self
    }

    pub fn value(&self) -> vk::WriteDescriptorSet {

        vk::WriteDescriptorSet {
            descriptor_count: self.writes.len() as _,
            p_buffer_info   : self.writes.as_ptr(),
            ..self.wi
        }
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::WriteDescriptorSet(for vk::Image).
#[derive(Debug, Clone)]
pub struct DescriptorImageSetWI {
    wi: vk::WriteDescriptorSet,
    writes: Vec<vk::DescriptorImageInfo>,
}

impl VulkanCI for DescriptorImageSetWI {
    type CIType = vk::WriteDescriptorSet;

    fn default_ci() -> Self::CIType {

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

impl DescriptorImageSetWI {

    pub fn new(set: vk::DescriptorSet, binding: vkuint, r#type: vk::DescriptorType) -> DescriptorImageSetWI {

        DescriptorImageSetWI {
            wi: vk::WriteDescriptorSet {
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
        self.writes.push(info); self
    }

    #[inline(always)]
    pub fn set_images(&mut self, infos: Vec<vk::DescriptorImageInfo>) {
        self.writes = infos;
    }

    #[inline(always)]
    pub fn dst_array_element(mut self, array_element: vkuint) -> DescriptorImageSetWI {
        self.wi.dst_array_element = array_element; self
    }

    pub fn value(&self) -> vk::WriteDescriptorSet {

        vk::WriteDescriptorSet {
            descriptor_count: self.writes.len() as _,
            p_image_info    : self.writes.as_ptr(),
            ..self.wi
        }
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
/// Convenient struct to update descriptor set.
#[derive(Default)]
pub struct DescriptorSetsUpdateCI {
    writes: Vec<vk::WriteDescriptorSet>,
    copies: Vec<vk::CopyDescriptorSet>,
}

impl DescriptorSetsUpdateCI {

    #[inline(always)]
    pub fn new() -> DescriptorSetsUpdateCI {
        DescriptorSetsUpdateCI::default()
    }

    #[inline(always)]
    pub fn add_write(mut self, value: vk::WriteDescriptorSet) -> DescriptorSetsUpdateCI {
        self.writes.push(value); self
    }

    #[inline(always)]
    pub fn add_copy(mut self, value: vk::CopyDescriptorSet) -> DescriptorSetsUpdateCI {
        self.copies.push(value); self
    }

    #[inline(always)]
    pub fn update(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.update_descriptor_sets(&self.writes, &self.copies);
        }
    }
}
// ----------------------------------------------------------------------------------------------

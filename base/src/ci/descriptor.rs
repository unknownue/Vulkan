
use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::VkDevice;
use crate::context::{VkObjectCreatable, VkObjectAllocatable};
use crate::ci::VulkanCI;
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

    pub fn flags(mut self, flags: vk::DescriptorPoolCreateFlags) -> DescriptorPoolCI {
        self.ci.flags = flags; self
    }

    pub fn add_descriptor(mut self, r#type: vk::DescriptorType, count: vkuint) -> DescriptorPoolCI {
        let new_descriptor = vk::DescriptorPoolSize {
            ty: r#type,
            descriptor_count: count,
        };
        self.pool_sizes.push(new_descriptor); self
    }

    pub fn build(mut self, device: &VkDevice) -> VkResult<vk::DescriptorPool> {

        self.ci.pool_size_count = self.pool_sizes.len() as _;
        self.ci.p_pool_sizes    = self.pool_sizes.as_ptr();

        let descriptor_pool = unsafe {
            device.logic.handle.create_descriptor_pool(&self.ci, None)
                .map_err(|_| VkError::create("Descriptor Pool"))?
        };
        Ok(descriptor_pool)
    }
}

impl VkObjectCreatable for vk::DescriptorPool {

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

impl DescriptorSetLayoutCI {

    pub fn new() -> DescriptorSetLayoutCI {

        DescriptorSetLayoutCI {
            ci: DescriptorSetLayoutCI::default_ci(),
            bindings: Vec::new(),
        }
    }

    pub fn build(mut self, device: &VkDevice) -> VkResult<vk::DescriptorSetLayout> {

        self.ci.binding_count = self.bindings.len() as _;
        self.ci.p_bindings    = self.bindings.as_ptr();

        let descriptor_set_layout = unsafe {
            device.logic.handle.create_descriptor_set_layout(&self.ci, None)
                .map_err(|_| VkError::create("Descriptor Set Layout"))?
        };
        Ok(descriptor_set_layout)
    }

    pub fn add_binding(mut self, binding: vk::DescriptorSetLayoutBinding) -> DescriptorSetLayoutCI {
        self.bindings.push(binding); self
    }

    pub fn flags(mut self, flags: vk::DescriptorSetLayoutCreateFlags) -> DescriptorSetLayoutCI {
        self.ci.flags = flags; self
    }
}

impl VkObjectCreatable for vk::DescriptorSetLayout {

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

    pub fn build(mut self, device: &VkDevice) -> VkResult<Vec<vk::DescriptorSet>> {

        self.ai.descriptor_set_count = self.set_layouts.len() as _;
        self.ai.p_set_layouts        = self.set_layouts.as_ptr();

        let descriptor_sets = unsafe {
            device.logic.handle.allocate_descriptor_sets(&self.ai)
                .map_err(|_| VkError::create("Allocate Descriptor Set"))?
        };
        Ok(descriptor_sets)
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

    pub fn add_buffer(mut self, info: vk::DescriptorBufferInfo) -> DescriptorBufferSetWI {
        self.writes.push(info); self
    }

    pub fn dst_array_element(mut self, array_element: vkuint) -> DescriptorBufferSetWI {
        self.wi.dst_array_element = array_element; self
    }

    pub fn build(&self) -> vk::WriteDescriptorSet {

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

impl VulkanCI<vk::WriteDescriptorSet> for DescriptorImageSetWI {

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

    pub fn add_image(mut self, info: vk::DescriptorImageInfo) -> DescriptorImageSetWI {
        self.writes.push(info); self
    }

    pub fn dst_array_element(mut self, array_element: vkuint) -> DescriptorImageSetWI {
        self.wi.dst_array_element = array_element; self
    }

    pub fn build(&self) -> vk::WriteDescriptorSet {

        vk::WriteDescriptorSet {
            descriptor_count: self.writes.len() as _,
            p_image_info    : self.writes.as_ptr(),
            ..self.wi
        }
    }
}
// ----------------------------------------------------------------------------------------------

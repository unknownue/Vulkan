
pub use self::renderpass::{RenderPassCI, RenderPassBI};
pub use self::renderpass::{AttachmentDescCI, SubpassDescCI, SubpassDependencyCI};

pub use self::state::VertexInputSCI;
pub use self::state::InputAssemblySCI;
pub use self::state::RasterizationSCI;
pub use self::state::{ColorBlendSCI, BlendAttachmentSCI};
pub use self::state::ViewportSCI;
pub use self::state::DepthStencilSCI;
pub use self::state::MultisampleSCI;
pub use self::state::DynamicSCI;

mod state;
mod renderpass;




use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::VkDevice;
use crate::context::VkObjectDiscardable;
use crate::ci::shader::ShaderStageCI;
use crate::ci::{VulkanCI, VkObjectBuildableCI};
use crate::error::{VkResult, VkError};
use crate::vkuint;

use std::ptr;

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::PipelineLayoutCreateInfo.
#[derive(Debug, Clone)]
pub struct PipelineLayoutCI {

    ci: vk::PipelineLayoutCreateInfo,
    set_layouts   : Vec<vk::DescriptorSetLayout>,
    push_constants: Vec<vk::PushConstantRange>,
}

impl VulkanCI for PipelineLayoutCI {
    type CIType = vk::PipelineLayoutCreateInfo;

    fn default_ci() -> Self::CIType {

        vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineLayoutCreateFlags::empty(),
            set_layout_count: 0,
            p_set_layouts   : ptr::null(),
            push_constant_range_count: 0,
            p_push_constant_ranges   : ptr::null(),
        }
    }
}

impl VkObjectBuildableCI for PipelineLayoutCI {
    type ObjectType = vk::PipelineLayout;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let pipeline_layout_ci = vk::PipelineLayoutCreateInfo {
            set_layout_count: self.set_layouts.len() as _,
            p_set_layouts   : self.set_layouts.as_ptr(),
            push_constant_range_count: self.push_constants.len() as _,
            p_push_constant_ranges   : self.push_constants.as_ptr(),
            ..self.ci
        };

        let pipeline_layout = unsafe {
            device.logic.handle.create_pipeline_layout(&pipeline_layout_ci, None)
                .map_err(|_| VkError::create("Pipeline Layout"))?
        };
        Ok(pipeline_layout)
    }
}

impl PipelineLayoutCI {

    pub fn new() -> PipelineLayoutCI {

        PipelineLayoutCI {
            ci: PipelineLayoutCI::default_ci(),
            set_layouts    : Vec::new(),
            push_constants : Vec::new(),
        }
    }

    pub fn add_set_layout(mut self, set_layout: vk::DescriptorSetLayout) -> PipelineLayoutCI {
        self.set_layouts.push(set_layout); self
    }

    pub fn add_push_constants(mut self, range: vk::PushConstantRange) -> PipelineLayoutCI {
        self.push_constants.push(range); self
    }

    pub fn flags(mut self, flags: vk::PipelineLayoutCreateFlags) -> PipelineLayoutCI {
        self.ci.flags = flags; self
    }
}

impl VkObjectDiscardable for vk::PipelineLayout {

    fn discard(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_pipeline_layout(self, None);
        }
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::FramebufferCreateInfo.
#[derive(Debug, Clone)]
pub struct FramebufferCI {

    ci: vk::FramebufferCreateInfo,
    attachments: Vec<vk::ImageView>,
}

impl VulkanCI for FramebufferCI {
    type CIType = vk::FramebufferCreateInfo;

    fn default_ci() -> Self::CIType {

        vk::FramebufferCreateInfo {
            s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
            p_next: ptr::null(),
            flags : vk::FramebufferCreateFlags::empty(),
            attachment_count: 0,
            p_attachments   : ptr::null(),
            width : 0,
            height: 0,
            layers: 1,
            render_pass: vk::RenderPass::null(),
        }
    }
}

impl VkObjectBuildableCI for FramebufferCI {
    type ObjectType = vk::Framebuffer;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let framebuffer_ci = vk::FramebufferCreateInfo {
            attachment_count: self.attachments.len() as _,
            p_attachments   : self.attachments.as_ptr(),
            ..self.ci
        };

        let framebuffer = unsafe {
            device.logic.handle.create_framebuffer(&framebuffer_ci, None)
                .map_err(|_| VkError::create("Framebuffer"))?
        };
        Ok(framebuffer)
    }
}

impl FramebufferCI {

    pub fn new(render_pass: vk::RenderPass, dimension: vk::Extent3D) -> FramebufferCI {

        FramebufferCI {
            ci: vk::FramebufferCreateInfo {
                render_pass,
                width : dimension.width,
                height: dimension.height,
                layers: dimension.depth,
                ..FramebufferCI::default_ci()
            },
            attachments: Vec::new(),
        }
    }

    pub fn new_2d(render_pass: vk::RenderPass, dimension: vk::Extent2D) -> FramebufferCI {

        let extent = vk::Extent3D {
            width : dimension.width,
            height: dimension.height,
            depth : 1,
        };
        FramebufferCI::new(render_pass, extent)
    }

    pub fn add_attachment(mut self, attachment: vk::ImageView) -> FramebufferCI {
        self.attachments.push(attachment); self
    }

    pub fn flags(mut self, flags: vk::FramebufferCreateFlags) -> FramebufferCI {
        self.ci.flags = flags; self
    }
}

impl VkObjectDiscardable for vk::Framebuffer {

    fn discard(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_framebuffer(self, None);
        }
    }
}

impl VkObjectDiscardable for &Vec<vk::Framebuffer> {

    fn discard(self, device: &VkDevice) {

        for framebuffer in self {
            device.discard(*framebuffer);
        }
    }
}
// ---------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
// Wrapper class for vk::GraphicsPipelineCreateInfo.
#[derive(Debug)]
pub struct GraphicsPipelineCI {
    ci: vk::GraphicsPipelineCreateInfo,

    shader_stages: Vec<ShaderStageCI>,

    vertex_input   : VertexInputSCI,
    input_assembly : InputAssemblySCI,
    rasterization  : RasterizationSCI,
    color_blend    : ColorBlendSCI,
    viewport       : ViewportSCI,
    depth_stencil  : DepthStencilSCI,
    multisample    : MultisampleSCI,
    dynamic        : DynamicSCI,

    cache: Option<vk::PipelineCache>,
}

impl VulkanCI for GraphicsPipelineCI {
    type CIType = vk::GraphicsPipelineCreateInfo;

    fn default_ci() -> Self::CIType {

        vk::GraphicsPipelineCreateInfo {
            s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
            p_next: ptr::null(),
            flags : vk::PipelineCreateFlags::empty(),
            layout: vk::PipelineLayout::null(),
            render_pass: vk::RenderPass::null(),
            stage_count            : 0,
            p_stages               : ptr::null(),
            p_vertex_input_state   : ptr::null(),
            p_input_assembly_state : ptr::null(),
            p_tessellation_state   : ptr::null(),
            p_viewport_state       : ptr::null(),
            p_rasterization_state  : ptr::null(),
            p_multisample_state    : ptr::null(),
            p_depth_stencil_state  : ptr::null(),
            p_color_blend_state    : ptr::null(),
            p_dynamic_state        : ptr::null(),
            subpass: 0,
            base_pipeline_handle: vk::Pipeline::null(),
            base_pipeline_index: -1,
        }
    }
}

impl VkObjectBuildableCI for GraphicsPipelineCI {
    type ObjectType = vk::Pipeline;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let stages: Vec<vk::PipelineShaderStageCreateInfo> = self.shader_stages.iter()
            .map(|s| s.value()).collect();
        let vertex_input = self.vertex_input.value();
        let input_assembly = self.input_assembly.value();
        let viewport = self.viewport.value();
        let rasterization = self.rasterization.value();
        let multisample = self.multisample.value();
        let depth_stencil = self.depth_stencil.value();
        let color_blend = self.color_blend.value();
        let dynamics = self.dynamic.value();

        let pipeline_ci = vk::GraphicsPipelineCreateInfo {
            stage_count            : stages.len() as _,
            p_stages               : stages.as_ptr(),
            p_vertex_input_state   : &vertex_input,
            p_input_assembly_state : &input_assembly,
            p_tessellation_state   : ptr::null(), // this field is not cover yet.
            p_viewport_state       : &viewport,
            p_rasterization_state  : &rasterization,
            p_multisample_state    : &multisample,
            p_depth_stencil_state  : &depth_stencil,
            p_color_blend_state    : &color_blend,
            p_dynamic_state        : &dynamics,
            ..self.ci
        };

        let pipeline = unsafe {
            device.logic.handle.create_graphics_pipelines(self.cache.unwrap_or(device.pipeline_cache), &[pipeline_ci], None)
                .map_err(|_| VkError::create("Graphics Pipeline"))?
        }.remove(0);

        Ok(pipeline)
    }
}

impl GraphicsPipelineCI {

    pub fn new(pass: vk::RenderPass, pipeline_layout: vk::PipelineLayout) -> GraphicsPipelineCI {

        GraphicsPipelineCI {
            ci: vk::GraphicsPipelineCreateInfo {
                render_pass: pass,
                layout: pipeline_layout,
                ..GraphicsPipelineCI::default_ci()
            },
            shader_stages  : Vec::new(),
            vertex_input   : VertexInputSCI::new(),
            input_assembly : InputAssemblySCI::new(),
            rasterization  : RasterizationSCI::new(),
            color_blend    : ColorBlendSCI::new(),
            viewport       : ViewportSCI::new(),
            depth_stencil  : DepthStencilSCI::new(),
            multisample    : MultisampleSCI::new(),
            dynamic        : DynamicSCI::new(),
            cache: None,
        }
    }

    pub fn set_use_subpass(&mut self, subpass: vkuint) {
        self.ci.subpass = subpass
    }

    pub fn set_base_pipeline(&mut self, pipeline: vk::Pipeline) {
        self.ci.base_pipeline_handle = pipeline;
    }

    pub fn set_flags(&mut self, flags: vk::PipelineCreateFlags) {
        self.ci.flags = flags;
    }

    pub fn set_shaders(&mut self, cis: Vec<ShaderStageCI>) {
        self.shader_stages = cis;
    }

    pub fn set_vertex_input(&mut self, sci: VertexInputSCI) {
        self.vertex_input = sci;
    }

    pub fn set_input_assembly(&mut self, sci: InputAssemblySCI) {
        self.input_assembly = sci;
    }

    pub fn set_rasterization(&mut self, sci: RasterizationSCI) {
        self.rasterization = sci;
    }

    pub fn set_color_blend(&mut self, sci: ColorBlendSCI) {
        self.color_blend = sci;
    }

    pub fn set_viewport(&mut self, sci: ViewportSCI) {
        self.viewport = sci;
    }

    pub fn set_depth_stencil(&mut self, sci: DepthStencilSCI) {
        self.depth_stencil = sci;
    }

    pub fn set_multisample(&mut self, sci: MultisampleSCI) {
        self.multisample = sci;
    }

    pub fn set_dynamic(&mut self, sci: DynamicSCI) {
        self.dynamic = sci;
    }

    pub fn set_pipeline_cache(&mut self, cache: vk::PipelineCache) {
        self.cache = Some(cache);
    }
}

impl VkObjectDiscardable for vk::Pipeline {

    fn discard(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_pipeline(self, None);
        }
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
// Wrapper class for vk::PipelineCacheCreateInfo.
#[derive(Debug, Clone)]
pub struct PipelineCacheCI {
    ci: vk::PipelineCacheCreateInfo,
}

impl VulkanCI for PipelineCacheCI {
    type CIType = vk::PipelineCacheCreateInfo;

    fn default_ci() -> vk::PipelineCacheCreateInfo {

        vk::PipelineCacheCreateInfo {
            s_type: vk::StructureType::PIPELINE_CACHE_CREATE_INFO,
            p_next: ptr::null(),
            flags : vk::PipelineCacheCreateFlags::empty(),
            initial_data_size: 0,
            p_initial_data: ptr::null(),
        }
    }
}

impl PipelineCacheCI {

    pub fn new() -> PipelineCacheCI {
        PipelineCacheCI {
            ci: PipelineCacheCI::default_ci(),
        }
    }

    pub fn flags(mut self, flags: vk::PipelineCacheCreateFlags) -> PipelineCacheCI {
        self.ci.flags = flags; self
    }

    pub fn build(&self, device: &VkDevice) -> VkResult<vk::PipelineCache> {
        unsafe {
            device.logic.handle.create_pipeline_cache(&self.ci, None)
                .map_err(|_| VkError::create("Graphics Cache"))
        }
    }
}

impl VkObjectDiscardable for vk::PipelineCache {

    fn discard(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_pipeline_cache(self, None);
        }
    }
}
// ----------------------------------------------------------------------------------------------


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

    inner: vk::PipelineLayoutCreateInfo,
    set_layouts   : Option<Vec<vk::DescriptorSetLayout>>,
    push_constants: Option<Vec<vk::PushConstantRange>>,
}

impl VulkanCI<vk::PipelineLayoutCreateInfo> for PipelineLayoutCI {

    fn default_ci() -> vk::PipelineLayoutCreateInfo {

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

impl AsRef<vk::PipelineLayoutCreateInfo> for PipelineLayoutCI {

    fn as_ref(&self) -> &vk::PipelineLayoutCreateInfo {
        &self.inner
    }
}

impl VkObjectBuildableCI for PipelineLayoutCI {
    type ObjectType = vk::PipelineLayout;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let pipeline_layout = unsafe {
            device.logic.handle.create_pipeline_layout(self.as_ref(), None)
                .map_err(|_| VkError::create("Pipeline Layout"))?
        };
        Ok(pipeline_layout)
    }
}

impl PipelineLayoutCI {

    pub fn new() -> PipelineLayoutCI {

        PipelineLayoutCI {
            inner: PipelineLayoutCI::default_ci(),
            set_layouts    : None,
            push_constants : None,
        }
    }

    #[inline(always)]
    pub fn add_set_layout(mut self, set_layout: vk::DescriptorSetLayout) -> PipelineLayoutCI {

        let set_layouts = self.set_layouts.get_or_insert(Vec::new());
        set_layouts.push(set_layout);

        self.inner.set_layout_count = set_layouts.len() as _;
        self.inner.p_set_layouts    = set_layouts.as_ptr(); self
    }

    #[inline(always)]
    pub fn add_push_constants(mut self, range: vk::PushConstantRange) -> PipelineLayoutCI {

        let push_constants = self.push_constants.get_or_insert(Vec::new());
        push_constants.push(range);

        self.inner.push_constant_range_count = push_constants.len() as _;
        self.inner.p_push_constant_ranges    = push_constants.as_ptr(); self
    }

    #[inline(always)]
    pub fn flags(mut self, flags: vk::PipelineLayoutCreateFlags) -> PipelineLayoutCI {
        self.inner.flags = flags; self
    }
}

impl VkObjectDiscardable for vk::PipelineLayout {

    fn discard_by(self, device: &VkDevice) {
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

    inner: vk::FramebufferCreateInfo,
    attachments: Vec<vk::ImageView>,
}

impl VulkanCI<vk::FramebufferCreateInfo> for FramebufferCI {

    fn default_ci() -> vk::FramebufferCreateInfo {

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

impl AsRef<vk::FramebufferCreateInfo> for FramebufferCI {

    fn as_ref(&self) -> &vk::FramebufferCreateInfo {
        &self.inner
    }
}

impl VkObjectBuildableCI for FramebufferCI {
    type ObjectType = vk::Framebuffer;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let framebuffer = unsafe {
            device.logic.handle.create_framebuffer(self.as_ref(), None)
                .map_err(|_| VkError::create("Framebuffer"))?
        };
        Ok(framebuffer)
    }
}

impl FramebufferCI {

    pub fn new(render_pass: vk::RenderPass, dimension: vk::Extent3D) -> FramebufferCI {

        FramebufferCI {
            inner: vk::FramebufferCreateInfo {
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

    #[inline(always)]
    pub fn add_attachment(mut self, attachment: vk::ImageView) -> FramebufferCI {

        self.attachments.push(attachment);
        self.inner.attachment_count = self.attachments.len() as _;
        self.inner.p_attachments    = self.attachments.as_ptr(); self
    }

    #[inline(always)]
    pub fn flags(mut self, flags: vk::FramebufferCreateFlags) -> FramebufferCI {
        self.inner.flags = flags; self
    }
}

impl VkObjectDiscardable for vk::Framebuffer {

    fn discard_by(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_framebuffer(self, None);
        }
    }
}

impl VkObjectDiscardable for &Vec<vk::Framebuffer> {

    fn discard_by(self, device: &VkDevice) {

        for framebuffer in self {
            device.discard(*framebuffer);
        }
    }
}
// ---------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
// Wrapper class for vk::GraphicsPipelineCreateInfo.
#[derive(Debug)]
pub struct GraphicsPipelineCI<'a> {

    inner: vk::GraphicsPipelineCreateInfo,

    vertex_input   : VertexInputSCI,
    input_assembly : InputAssemblySCI,
    rasterization  : RasterizationSCI,
    color_blend    : ColorBlendSCI,
    viewport       : ViewportSCI,
    depth_stencil  : DepthStencilSCI,
    multisample    : MultisampleSCI,
    dynamics       : DynamicSCI,

    cache: Option<vk::PipelineCache>,
    shader_stages: Vec<vk::PipelineShaderStageCreateInfo>,

    phantom_type: ::std::marker::PhantomData<&'a ()>,
}

impl<'a> VkObjectBuildableCI for GraphicsPipelineCI<'a> {
    type ObjectType = vk::Pipeline;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let pipeline_ci = vk::GraphicsPipelineCreateInfo {
            stage_count            : self.shader_stages.len() as _,
            p_stages               : self.shader_stages.as_ptr(),
            p_vertex_input_state   : self.vertex_input.as_ref(),
            p_input_assembly_state : self.input_assembly.as_ref(),
            p_tessellation_state   : ptr::null(), // this field is not cover yet.
            p_viewport_state       : self.viewport.as_ref(),
            p_rasterization_state  : self.rasterization.as_ref(),
            p_multisample_state    : self.multisample.as_ref(),
            p_depth_stencil_state  : self.depth_stencil.as_ref(),
            p_color_blend_state    : self.color_blend.as_ref(),
            p_dynamic_state        : self.dynamics.as_ref(),
            ..self.inner
        };

        let pipeline = unsafe {
            device.logic.handle.create_graphics_pipelines(self.cache.unwrap_or(device.pipeline_cache), &[pipeline_ci], None)
                .map_err(|_| VkError::create("Graphics Pipeline"))?
        }.remove(0);

        Ok(pipeline)
    }
}

impl<'b, 'a: 'b> GraphicsPipelineCI<'a> {

    pub fn new(pass: vk::RenderPass, pipeline_layout: vk::PipelineLayout) -> GraphicsPipelineCI<'a> {

        GraphicsPipelineCI {
            inner: vk::GraphicsPipelineCreateInfo {
                render_pass: pass,
                layout: pipeline_layout,
                base_pipeline_index: -1,
                ..Default::default()
            },
            shader_stages  : Vec::new(),
            vertex_input   : VertexInputSCI::new(),
            input_assembly : InputAssemblySCI::new(),
            rasterization  : RasterizationSCI::new(),
            color_blend    : ColorBlendSCI::new(),
            viewport       : ViewportSCI::new(),
            depth_stencil  : DepthStencilSCI::new(),
            multisample    : MultisampleSCI::new(),
            dynamics       : DynamicSCI::new(),
            cache: None,
            phantom_type: ::std::marker::PhantomData,
        }
    }

    #[inline(always)]
    pub fn set_use_subpass(&mut self, subpass: vkuint) {
        self.inner.subpass = subpass
    }

    #[inline(always)]
    pub fn set_base_pipeline(&mut self, pipeline: vk::Pipeline) {
        self.inner.base_pipeline_handle = pipeline;
    }

    #[inline(always)]
    pub fn set_flags(&mut self, flags: vk::PipelineCreateFlags) {
        self.inner.flags = flags;
    }

    #[inline(always)]
    pub fn set_shaders(&mut self, cis: &'b [ShaderStageCI]) {

        self.shader_stages = cis.iter()
            .map(|s| s.as_ref().clone())
            .collect();
    }

    #[inline(always)]
    pub fn set_vertex_input(&mut self, sci: VertexInputSCI) {
        self.vertex_input = sci;
    }

    #[inline(always)]
    pub fn set_input_assembly(&mut self, sci: InputAssemblySCI) {
        self.input_assembly = sci;
    }

    #[inline(always)]
    pub fn set_rasterization(&mut self, sci: RasterizationSCI) {
        self.rasterization = sci;
    }

    #[inline(always)]
    pub fn set_color_blend(&mut self, sci: ColorBlendSCI) {
        self.color_blend = sci;
    }

    #[inline(always)]
    pub fn set_viewport(&mut self, sci: ViewportSCI) {
        self.viewport = sci;
    }

    #[inline(always)]
    pub fn set_depth_stencil(&mut self, sci: DepthStencilSCI) {
        self.depth_stencil = sci;
    }

    #[inline(always)]
    pub fn set_multisample(&mut self, sci: MultisampleSCI) {
        self.multisample = sci;
    }

    #[inline(always)]
    pub fn set_dynamic(&mut self, sci: DynamicSCI) {
        self.dynamics = sci;
    }

    #[inline(always)]
    pub fn set_pipeline_cache(&mut self, cache: vk::PipelineCache) {
        self.cache = Some(cache);
    }
}

impl VkObjectDiscardable for vk::Pipeline {

    fn discard_by(self, device: &VkDevice) {
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
    inner: vk::PipelineCacheCreateInfo,
}

impl VulkanCI<vk::PipelineCacheCreateInfo> for PipelineCacheCI {

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

impl AsRef<vk::PipelineCacheCreateInfo> for PipelineCacheCI {

    fn as_ref(&self) -> &vk::PipelineCacheCreateInfo {
        &self.inner
    }
}

impl PipelineCacheCI {

    #[inline(always)]
    pub fn new() -> PipelineCacheCI {
        PipelineCacheCI {
            inner: PipelineCacheCI::default_ci(),
        }
    }

    #[inline(always)]
    pub fn flags(mut self, flags: vk::PipelineCacheCreateFlags) -> PipelineCacheCI {
        self.inner.flags = flags; self
    }

    pub fn build(&self, device: &VkDevice) -> VkResult<vk::PipelineCache> {
        unsafe {
            device.logic.handle.create_pipeline_cache(self.as_ref(), None)
                .map_err(|_| VkError::create("Graphics Cache"))
        }
    }
}

impl VkObjectDiscardable for vk::PipelineCache {

    fn discard_by(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_pipeline_cache(self, None);
        }
    }
}
// ----------------------------------------------------------------------------------------------

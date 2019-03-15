
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
use std::ops::Deref;

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

impl Deref for PipelineLayoutCI {
    type Target = vk::PipelineLayoutCreateInfo;

    fn deref(&self) -> &vk::PipelineLayoutCreateInfo {
        &self.inner
    }
}

impl VkObjectBuildableCI for PipelineLayoutCI {
    type ObjectType = vk::PipelineLayout;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let pipeline_layout = unsafe {
            device.logic.handle.create_pipeline_layout(self, None)
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

impl Deref for FramebufferCI {
    type Target = vk::FramebufferCreateInfo;

    fn deref(&self) -> &vk::FramebufferCreateInfo {
        &self.inner
    }
}

impl VkObjectBuildableCI for FramebufferCI {
    type ObjectType = vk::Framebuffer;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let framebuffer = unsafe {
            device.logic.handle.create_framebuffer(self, None)
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
    dynamic        : DynamicSCI,

    cache: Option<vk::PipelineCache>,

    phantom_type: ::std::marker::PhantomData<&'a ()>,
}

impl<'a> VulkanCI<vk::GraphicsPipelineCreateInfo> for GraphicsPipelineCI<'a> {

    fn default_ci() -> vk::GraphicsPipelineCreateInfo {

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
            base_pipeline_index : -1,
        }
    }
}

impl<'a> Deref for GraphicsPipelineCI<'a> {
    type Target = vk::GraphicsPipelineCreateInfo;

    fn deref(&self) -> &vk::GraphicsPipelineCreateInfo {
        &self.inner
    }
}

impl<'a> VkObjectBuildableCI for GraphicsPipelineCI<'a> {
    type ObjectType = vk::Pipeline;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let pipeline = unsafe {
            device.logic.handle.create_graphics_pipelines(self.cache.unwrap_or(device.pipeline_cache), &[self.inner], None)
                .map_err(|_| VkError::create("Graphics Pipeline"))?
        }.remove(0);

        Ok(pipeline)
    }
}

impl<'a, 'b> GraphicsPipelineCI<'a> {

    pub fn new(pass: vk::RenderPass, pipeline_layout: vk::PipelineLayout) -> GraphicsPipelineCI<'a> {

        let mut pipeline_ci = GraphicsPipelineCI {
            inner: vk::GraphicsPipelineCreateInfo {
                render_pass: pass,
                layout: pipeline_layout,
                ..GraphicsPipelineCI::default_ci()
            },
            vertex_input   : VertexInputSCI::new(),
            input_assembly : InputAssemblySCI::new(),
            rasterization  : RasterizationSCI::new(),
            color_blend    : ColorBlendSCI::new(),
            viewport       : ViewportSCI::new(),
            depth_stencil  : DepthStencilSCI::new(),
            multisample    : MultisampleSCI::new(),
            dynamic        : DynamicSCI::new(),
            cache: None,
            phantom_type: ::std::marker::PhantomData,
        };

        pipeline_ci.inner.p_vertex_input_state   = pipeline_ci.vertex_input.deref();
        pipeline_ci.inner.p_input_assembly_state = pipeline_ci.input_assembly.deref();
        pipeline_ci.inner.p_rasterization_state  = pipeline_ci.rasterization.deref();
        pipeline_ci.inner.p_color_blend_state    = pipeline_ci.color_blend.deref();
        pipeline_ci.inner.p_viewport_state       = pipeline_ci.viewport.deref();
        pipeline_ci.inner.p_depth_stencil_state  = pipeline_ci.depth_stencil.deref();
        pipeline_ci.inner.p_multisample_state    = pipeline_ci.multisample.deref();
        pipeline_ci.inner.p_dynamic_state        = pipeline_ci.dynamic.deref();

        pipeline_ci
    }

    #[inline(always)]
    pub fn set_use_subpass(&mut self, subpass: vkuint) {
        self.inner.subpass = subpass;
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

        let stages: Vec<vk::PipelineShaderStageCreateInfo> = cis.iter()
            .map(|s| s.deref().clone()).collect();

        self.inner.stage_count = stages.len() as _;
        self.inner.p_stages    = stages.as_ptr();
    }

    #[inline(always)]
    pub fn set_vertex_input(&mut self, sci: VertexInputSCI) {
        self.inner.p_vertex_input_state = sci.deref();
        self.vertex_input = sci;
    }

    #[inline(always)]
    pub fn set_input_assembly(&mut self, sci: InputAssemblySCI) {
        self.inner.p_input_assembly_state = sci.deref();
        self.input_assembly = sci;
    }

    #[inline(always)]
    pub fn set_rasterization(&mut self, sci: RasterizationSCI) {
        self.inner.p_rasterization_state = sci.deref();
        self.rasterization = sci;
    }

    #[inline(always)]
    pub fn set_color_blend(&mut self, sci: ColorBlendSCI) {
        self.inner.p_color_blend_state = sci.deref();
        self.color_blend = sci;
    }

    #[inline(always)]
    pub fn set_viewport(&mut self, sci: ViewportSCI) {
        self.inner.p_viewport_state = sci.deref();
        self.viewport = sci;
    }

    #[inline(always)]
    pub fn set_depth_stencil(&mut self, sci: DepthStencilSCI) {
        self.inner.p_depth_stencil_state = sci.deref();
        self.depth_stencil = sci;
    }

    #[inline(always)]
    pub fn set_multisample(&mut self, sci: MultisampleSCI) {
        self.inner.p_multisample_state = sci.deref();
        self.multisample = sci;
    }

    #[inline(always)]
    pub fn set_dynamic(&mut self, sci: DynamicSCI) {
        self.inner.p_dynamic_state = sci.deref();
        self.dynamic = sci;
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

impl Deref for PipelineCacheCI {
    type Target = vk::PipelineCacheCreateInfo;

    fn deref(&self) -> &vk::PipelineCacheCreateInfo {
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
            device.logic.handle.create_pipeline_cache(self, None)
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

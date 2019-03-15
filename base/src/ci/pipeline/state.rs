
use ash::vk;

use crate::ci::VulkanCI;
use crate::vkfloat;

use std::ptr;
use std::ops::Deref;

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::PipelineVertexInputStateCreateInfo.
#[derive(Debug, Clone)]
pub struct VertexInputSCI {

    inner: vk::PipelineVertexInputStateCreateInfo,
    bindings  : Vec<vk::VertexInputBindingDescription>,
    attributes: Vec<vk::VertexInputAttributeDescription>,
}

impl VulkanCI<vk::PipelineVertexInputStateCreateInfo> for VertexInputSCI {

    fn default_ci() -> vk::PipelineVertexInputStateCreateInfo {

        vk::PipelineVertexInputStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags : vk::PipelineVertexInputStateCreateFlags::empty(),
            vertex_binding_description_count: 0,
            p_vertex_binding_descriptions   : ptr::null(),
            vertex_attribute_description_count: 0,
            p_vertex_attribute_descriptions   : ptr::null(),
        }
    }
}

impl Deref for VertexInputSCI {
    type Target = vk::PipelineVertexInputStateCreateInfo;

    fn deref(&self) -> &vk::PipelineVertexInputStateCreateInfo {
        &self.inner
    }
}

impl VertexInputSCI {

    pub fn new() -> VertexInputSCI {

        VertexInputSCI {
            inner: VertexInputSCI::default_ci(),
            bindings  : Vec::new(),
            attributes: Vec::new(),
        }
    }

    #[inline(always)]
    pub fn add_binding(mut self, binding: vk::VertexInputBindingDescription) -> VertexInputSCI {

        self.bindings.push(binding);
        self.inner.vertex_binding_description_count = self.bindings.len() as _;
        self.inner.p_vertex_binding_descriptions    = self.bindings.as_ptr(); self
    }

    #[inline(always)]
    pub fn add_attribute(mut self, attribute: vk::VertexInputAttributeDescription) -> VertexInputSCI {

        self.attributes.push(attribute);
        self.inner.vertex_attribute_description_count = self.attributes.len() as _;
        self.inner.p_vertex_attribute_descriptions    = self.attributes.as_ptr(); self
    }

    #[inline(always)]
    pub fn flags(mut self, flags: vk::PipelineVertexInputStateCreateFlags) -> VertexInputSCI {
        self.inner.flags = flags; self
    }

    // For crate inner use.
    #[doc(hidden)]
    pub(crate) fn inner_set_attribute_locations(&mut self) {
        for (i, attribute) in self.attributes.iter_mut().enumerate() {
            attribute.location = i as _;
        }
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::PipelineInputAssemblyStateCreateInfo.
#[derive(Debug, Clone)]
pub struct InputAssemblySCI {
    inner: vk::PipelineInputAssemblyStateCreateInfo,
}

impl VulkanCI<vk::PipelineInputAssemblyStateCreateInfo> for InputAssemblySCI {

    fn default_ci() -> vk::PipelineInputAssemblyStateCreateInfo {

        vk::PipelineInputAssemblyStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags : vk::PipelineInputAssemblyStateCreateFlags::empty(),
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: vk::FALSE,
        }
    }
}

impl Deref for InputAssemblySCI {
    type Target = vk::PipelineInputAssemblyStateCreateInfo;

    fn deref(&self) -> &vk::PipelineInputAssemblyStateCreateInfo {
        &self.inner
    }
}

impl InputAssemblySCI {

    #[inline(always)]
    pub fn new() -> InputAssemblySCI {

        InputAssemblySCI {
            inner: InputAssemblySCI::default_ci(),
        }
    }

    #[inline(always)]
    pub fn topology(mut self, topology: vk::PrimitiveTopology) -> InputAssemblySCI {
        self.inner.topology = topology; self
    }

    #[inline(always)]
    pub fn primitive_restart(mut self, is_enable: bool) -> InputAssemblySCI {
        self.inner.primitive_restart_enable = if is_enable { vk::TRUE } else { vk::FALSE }; self
    }

    #[inline(always)]
    pub fn flags(mut self, flags: vk::PipelineInputAssemblyStateCreateFlags) -> InputAssemblySCI {
        self.inner.flags = flags; self
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::PipelineRasterizationStateCreateInfo.
#[derive(Debug, Clone)]
pub struct RasterizationSCI {
    inner: vk::PipelineRasterizationStateCreateInfo,
}

impl VulkanCI<vk::PipelineRasterizationStateCreateInfo> for RasterizationSCI {

    fn default_ci() -> vk::PipelineRasterizationStateCreateInfo {

        vk::PipelineRasterizationStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags : vk::PipelineRasterizationStateCreateFlags::empty(),
            depth_clamp_enable         : vk::FALSE,
            rasterizer_discard_enable  : vk::FALSE,
            polygon_mode               : vk::PolygonMode::FILL,
            cull_mode                  : vk::CullModeFlags::NONE,
            front_face                 : vk::FrontFace::COUNTER_CLOCKWISE,
            depth_bias_enable          : vk::FALSE,
            depth_bias_constant_factor : 0.0,
            depth_bias_clamp           : 0.0,
            depth_bias_slope_factor    : 0.0,
            line_width                 : 1.0,
        }
    }
}

impl Deref for RasterizationSCI {
    type Target = vk::PipelineRasterizationStateCreateInfo;

    fn deref(&self) -> &vk::PipelineRasterizationStateCreateInfo {
        &self.inner
    }
}

impl RasterizationSCI {

    #[inline(always)]
    pub fn new() -> RasterizationSCI {

        RasterizationSCI {
            inner: RasterizationSCI::default_ci(),
        }
    }

    #[inline(always)]
    pub fn depth_clamp(mut self, is_enable: bool, bias: vkfloat) -> RasterizationSCI {
        self.inner.depth_clamp_enable = if is_enable { vk::TRUE } else { vk::FALSE };
        self.inner.depth_bias_clamp = bias; self
    }

    #[inline(always)]
    pub fn rasterizer_discard(mut self, is_enable: bool) -> RasterizationSCI {
        self.inner.rasterizer_discard_enable = if is_enable { vk::TRUE } else { vk::FALSE }; self
    }

    #[inline(always)]
    pub fn cull_face(mut self, mode: vk::CullModeFlags, front_face: vk::FrontFace) -> RasterizationSCI {
        self.inner.cull_mode = mode;
        self.inner.front_face = front_face; self
    }

    #[inline(always)]
    pub fn polygon(mut self, mode: vk::PolygonMode) -> RasterizationSCI {
        self.inner.polygon_mode = mode; self
    }

    #[inline(always)]
    pub fn depth_bias(mut self, is_enable: bool, constant_factor: vkfloat, slope_factor: vkfloat) -> RasterizationSCI {
        self.inner.depth_bias_enable = if is_enable { vk::TRUE } else { vk::FALSE };
        self.inner.depth_bias_constant_factor = constant_factor;
        self.inner.depth_bias_slope_factor = slope_factor; self
    }

    #[inline(always)]
    pub fn line_width(mut self, width: vkfloat) -> RasterizationSCI {
        self.inner.line_width = width; self
    }
}
// ----------------------------------------------------------------------------------------------


// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::PipelineColorBlendStateCreateInfo.
#[derive(Debug, Clone)]
pub struct ColorBlendSCI {
    inner: vk::PipelineColorBlendStateCreateInfo,
    attachments: Vec<vk::PipelineColorBlendAttachmentState>,
}

impl VulkanCI<vk::PipelineColorBlendStateCreateInfo> for ColorBlendSCI {

    fn default_ci() -> vk::PipelineColorBlendStateCreateInfo {

        vk::PipelineColorBlendStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags : vk::PipelineColorBlendStateCreateFlags::empty(),
            logic_op_enable: vk::FALSE,
            logic_op       : vk::LogicOp::COPY,
            attachment_count: 0,
            p_attachments   : ptr::null(),
            blend_constants : [0.0; 4],
        }
    }
}

impl Deref for ColorBlendSCI {
    type Target = vk::PipelineColorBlendStateCreateInfo;

    fn deref(&self) -> &vk::PipelineColorBlendStateCreateInfo {
        &self.inner
    }
}

impl ColorBlendSCI {

    pub fn new() -> ColorBlendSCI {

        ColorBlendSCI {
            inner: ColorBlendSCI::default_ci(),
            attachments: Vec::new(),
        }
    }

    #[inline(always)]
    pub fn add_attachment(mut self, attachment: BlendAttachmentSCI) -> ColorBlendSCI {

        self.attachments.push(attachment.into());
        self.inner.attachment_count = self.attachments.len() as _;
        self.inner.p_attachments    = self.attachments.as_ptr(); self
    }

    #[inline(always)]
    pub fn logic_op(mut self, is_enable: bool, op: vk::LogicOp) -> ColorBlendSCI {
        self.inner.logic_op_enable = if is_enable { vk::TRUE } else { vk::FALSE };
        self.inner.logic_op = op; self
    }

    #[inline(always)]
    pub fn blend_constants(mut self, constants: [vkfloat; 4]) -> ColorBlendSCI {
        self.inner.blend_constants = constants; self
    }

    #[inline(always)]
    pub fn flags(mut self, flags: vk::PipelineColorBlendStateCreateFlags) -> ColorBlendSCI {
        self.inner.flags = flags; self
    }
}

/// Wrapper class for vk::PipelineColorBlendStateCreateInfo.
#[derive(Debug, Clone)]
pub struct BlendAttachmentSCI {
    inner: vk::PipelineColorBlendAttachmentState,
}

impl VulkanCI<vk::PipelineColorBlendAttachmentState> for BlendAttachmentSCI {

    fn default_ci() -> vk::PipelineColorBlendAttachmentState {

        vk::PipelineColorBlendAttachmentState {
            blend_enable: vk::FALSE,
            src_color_blend_factor: vk::BlendFactor::ONE,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask:vk::ColorComponentFlags::R
                | vk::ColorComponentFlags::G
                | vk::ColorComponentFlags::B
                | vk::ColorComponentFlags::A,
        }
    }
}

impl Deref for BlendAttachmentSCI {
    type Target = vk::PipelineColorBlendAttachmentState;

    fn deref(&self) -> &vk::PipelineColorBlendAttachmentState {
        &self.inner
    }
}

impl BlendAttachmentSCI {

    #[inline(always)]
    pub fn new() -> BlendAttachmentSCI {

        BlendAttachmentSCI {
            inner: BlendAttachmentSCI::default_ci(),
        }
    }

    #[inline(always)]
    pub fn blend_enable(mut self, is_enable: bool) -> BlendAttachmentSCI {
        self.inner.blend_enable = if is_enable { vk::TRUE } else { vk::FALSE }; self
    }

    #[inline(always)]
    pub fn color(mut self, op: vk::BlendOp, src_factor: vk::BlendFactor, dst_factor: vk::BlendFactor) -> BlendAttachmentSCI {
        self.inner.src_color_blend_factor = src_factor;
        self.inner.dst_color_blend_factor = dst_factor;
        self.inner.color_blend_op = op; self
    }

    #[inline(always)]
    pub fn alpha(mut self, op: vk::BlendOp, src_factor: vk::BlendFactor, dst_factor: vk::BlendFactor) -> BlendAttachmentSCI {
        self.inner.src_alpha_blend_factor = src_factor;
        self.inner.dst_alpha_blend_factor = dst_factor;
        self.inner.alpha_blend_op = op; self
    }

    #[inline(always)]
    pub fn color_write_mask(mut self, mask: vk::ColorComponentFlags) -> BlendAttachmentSCI {
        self.inner.color_write_mask = mask; self
    }
}

impl From<BlendAttachmentSCI> for vk::PipelineColorBlendAttachmentState {

    fn from(v: BlendAttachmentSCI) -> vk::PipelineColorBlendAttachmentState {
        v.inner
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::PipelineViewportStateCreateInfo.
#[derive(Debug, Clone)]
pub struct ViewportSCI {

    inner: vk::PipelineViewportStateCreateInfo,
    viewports: Vec<vk::Viewport>,
    scissors : Vec<vk::Rect2D>,
}

impl VulkanCI<vk::PipelineViewportStateCreateInfo> for ViewportSCI {

    fn default_ci() -> vk::PipelineViewportStateCreateInfo {

        vk::PipelineViewportStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags : vk::PipelineViewportStateCreateFlags::empty(),
            viewport_count : 0,
            p_viewports    : ptr::null(),
            scissor_count  : 0,
            p_scissors     : ptr::null(),
        }
    }
}

impl Deref for ViewportSCI {
    type Target = vk::PipelineViewportStateCreateInfo;

    fn deref(&self) -> &vk::PipelineViewportStateCreateInfo {
        &self.inner
    }
}

impl ViewportSCI {

    pub fn new() -> ViewportSCI {

        ViewportSCI {
            inner: ViewportSCI::default_ci(),
            viewports: Vec::new(),
            scissors : Vec::new(),
        }
    }

    #[inline(always)]
    pub fn add_viewport(mut self, viewport: vk::Viewport) -> ViewportSCI {

        self.viewports.push(viewport);
        self.inner.viewport_count = self.viewports.len() as _;
        self.inner.p_viewports    = self.viewports.as_ptr(); self
    }

    #[inline(always)]
    pub fn add_scissor(mut self, scissor: vk::Rect2D) -> ViewportSCI {

        self.scissors.push(scissor);
        self.inner.scissor_count = self.scissors.len() as _;
        self.inner.p_scissors    = self.scissors.as_ptr(); self
    }

    #[inline(always)]
    pub fn flags(mut self, flags: vk::PipelineViewportStateCreateFlags) -> ViewportSCI {
        self.inner.flags = flags; self
    }
}
// ----------------------------------------------------------------------------------------------


// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::PipelineDepthStencilStateCreateInfo.
#[derive(Debug, Clone)]
pub struct DepthStencilSCI {
    inner: vk::PipelineDepthStencilStateCreateInfo,
}

impl VulkanCI<vk::PipelineDepthStencilStateCreateInfo> for DepthStencilSCI {

    fn default_ci() -> vk::PipelineDepthStencilStateCreateInfo {

        let stencil_op = vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op   : vk::CompareOp::ALWAYS,
            compare_mask : 0,
            write_mask   : 0,
            reference    : 0,
        };

        vk::PipelineDepthStencilStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags : vk::PipelineDepthStencilStateCreateFlags::empty(),
            depth_test_enable        : vk::FALSE,
            depth_write_enable       : vk::FALSE,
            depth_compare_op         : vk::CompareOp::LESS_OR_EQUAL,
            depth_bounds_test_enable : vk::FALSE,
            stencil_test_enable      : vk::FALSE,
            front: stencil_op,
            back : stencil_op,
            min_depth_bounds: 0.0,
            max_depth_bounds: 1.0,
        }
    }
}

impl Deref for DepthStencilSCI {
    type Target = vk::PipelineDepthStencilStateCreateInfo;

    fn deref(&self) -> &vk::PipelineDepthStencilStateCreateInfo {
        &self.inner
    }
}

impl DepthStencilSCI {

    #[inline(always)]
    pub fn new() -> DepthStencilSCI {

        DepthStencilSCI {
            inner: DepthStencilSCI::default_ci(),
        }
    }

    #[inline(always)]
    pub fn depth_test(mut self, is_enable_test: bool, is_enable_write: bool, compare_op: vk::CompareOp) -> DepthStencilSCI {
        self.inner.depth_test_enable = if is_enable_test { vk::TRUE } else { vk::FALSE };
        self.inner.depth_write_enable = if is_enable_write { vk::TRUE } else { vk::FALSE };
        self.inner.depth_compare_op = compare_op; self
    }

    #[inline(always)]
    pub fn depth_bounds(mut self, is_enable: bool, min: vkfloat, max: vkfloat) -> DepthStencilSCI {
        self.inner.depth_bounds_test_enable = if is_enable { vk::TRUE } else { vk::FALSE };
        self.inner.min_depth_bounds = min;
        self.inner.max_depth_bounds = max; self
    }

    #[inline(always)]
    pub fn stencil(mut self, is_enable: bool, front: vk::StencilOpState, back: vk::StencilOpState) -> DepthStencilSCI {
        self.inner.stencil_test_enable =  if is_enable { vk::TRUE } else { vk::FALSE };
        self.inner.front = front;
        self.inner.back  = back; self
    }

    #[inline(always)]
    pub fn flags(mut self, flags: vk::PipelineDepthStencilStateCreateFlags) -> DepthStencilSCI {
        self.inner.flags = flags; self
    }
}
// ----------------------------------------------------------------------------------------------



// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::PipelineMultisampleStateCreateInfo.
#[derive(Debug, Clone)]
pub struct MultisampleSCI {

    inner: vk::PipelineMultisampleStateCreateInfo,
    sample_mask: Option<vk::SampleMask>,
}

impl VulkanCI<vk::PipelineMultisampleStateCreateInfo> for MultisampleSCI {

    fn default_ci() -> vk::PipelineMultisampleStateCreateInfo {

        vk::PipelineMultisampleStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags : vk::PipelineMultisampleStateCreateFlags::empty(),
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            sample_shading_enable: vk::FALSE,
            min_sample_shading: 0.0,
            p_sample_mask: ptr::null(),
            alpha_to_coverage_enable: vk::FALSE,
            alpha_to_one_enable     : vk::FALSE,
        }
    }
}

impl Deref for MultisampleSCI {
    type Target = vk::PipelineMultisampleStateCreateInfo;

    fn deref(&self) -> &vk::PipelineMultisampleStateCreateInfo {
        &self.inner
    }
}

impl MultisampleSCI {

    pub fn new() -> MultisampleSCI {

        MultisampleSCI {
            inner: MultisampleSCI::default_ci(),
            sample_mask: None,
        }
    }

    #[inline(always)]
    pub fn sample_count(mut self, count: vk::SampleCountFlags) -> MultisampleSCI {
        self.inner.rasterization_samples = count; self
    }

    #[inline(always)]
    pub fn sample_shading(mut self, is_enable: bool, min: vkfloat) -> MultisampleSCI {
        self.inner.sample_shading_enable = if is_enable { vk::TRUE } else { vk::FALSE };
        self.inner.min_sample_shading = min; self
    }

    #[inline(always)]
    pub fn sample_mask(mut self, mask: vk::SampleMask) -> MultisampleSCI {
        self.inner.p_sample_mask = &mask;
        self.sample_mask = Some(mask); self
    }

    #[inline(always)]
    pub fn alpha(mut self, is_enable_alpha2coverage: bool, is_enable_alpha2one: bool) -> MultisampleSCI {
        self.inner.alpha_to_coverage_enable = if is_enable_alpha2coverage { vk::TRUE } else { vk::FALSE };
        self.inner.alpha_to_one_enable = if is_enable_alpha2one { vk::TRUE } else { vk::FALSE }; self
    }

    #[inline(always)]
    pub fn flags(mut self, flags: vk::PipelineMultisampleStateCreateFlags) -> MultisampleSCI {
        self.inner.flags = flags; self
    }
}
// ----------------------------------------------------------------------------------------------


// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::PipelineDynamicStateCreateInfo.
#[derive(Debug, Clone)]
pub struct DynamicSCI {

    inner: vk::PipelineDynamicStateCreateInfo,
    dynamics: Option<Vec<vk::DynamicState>>,
}

impl VulkanCI<vk::PipelineDynamicStateCreateInfo> for DynamicSCI {

    fn default_ci() -> vk::PipelineDynamicStateCreateInfo {

        vk::PipelineDynamicStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags : vk::PipelineDynamicStateCreateFlags::empty(),
            dynamic_state_count: 0,
            p_dynamic_states   : ptr::null(),
        }
    }
}

impl Deref for DynamicSCI {
    type Target = vk::PipelineDynamicStateCreateInfo;

    fn deref(&self) -> &vk::PipelineDynamicStateCreateInfo {
        &self.inner
    }
}

impl DynamicSCI {

    #[inline]
    pub fn new() -> DynamicSCI {

        DynamicSCI {
            inner: DynamicSCI::default_ci(),
            dynamics: None,
        }
    }

    #[inline]
    pub fn add_dynamic(mut self, state: vk::DynamicState) -> DynamicSCI {

        let dynamics = self.dynamics.get_or_insert(Vec::new());
        dynamics.push(state);

        self.inner.dynamic_state_count = dynamics.len() as _;
        self.inner.p_dynamic_states    = dynamics.as_ptr(); self
    }
}
// ----------------------------------------------------------------------------------------------


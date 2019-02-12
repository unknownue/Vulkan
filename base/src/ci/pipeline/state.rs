
use ash::vk;

use crate::ci::VulkanCI;
use crate::vkfloat;

use std::ptr;

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::PipelineVertexInputStateCreateInfo.
#[derive(Debug, Clone)]
pub struct VertexInputSCI {
    sci: vk::PipelineVertexInputStateCreateInfo,
    bindings  : Vec<vk::VertexInputBindingDescription>,
    attributes: Vec<vk::VertexInputAttributeDescription>,
}

impl VulkanCI for VertexInputSCI {
    type CIType = vk::PipelineVertexInputStateCreateInfo;

    fn default_ci() -> Self::CIType {

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

impl VertexInputSCI {

    pub fn new() -> VertexInputSCI {

        VertexInputSCI {
            sci: VertexInputSCI::default_ci(),
            bindings  : Vec::new(),
            attributes: Vec::new(),
        }
    }

    pub fn value(&self) -> vk::PipelineVertexInputStateCreateInfo {

        vk::PipelineVertexInputStateCreateInfo {
            vertex_binding_description_count   : self.bindings.len() as _,
            p_vertex_binding_descriptions      : self.bindings.as_ptr(),
            vertex_attribute_description_count : self.attributes.len() as _,
            p_vertex_attribute_descriptions    : self.attributes.as_ptr(),
            ..self.sci
        }
    }

    pub fn add_binding(mut self, binding: vk::VertexInputBindingDescription) -> VertexInputSCI {
        self.bindings.push(binding); self
    }

    pub fn add_attribute(mut self, attribute: vk::VertexInputAttributeDescription) -> VertexInputSCI {
        self.attributes.push(attribute); self
    }

    pub fn flags(mut self, flags: vk::PipelineVertexInputStateCreateFlags) -> VertexInputSCI {
        self.sci.flags = flags; self
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::PipelineInputAssemblyStateCreateInfo.
#[derive(Debug, Clone)]
pub struct InputAssemblySCI {
    sci: vk::PipelineInputAssemblyStateCreateInfo,
}

impl VulkanCI for InputAssemblySCI {
    type CIType = vk::PipelineInputAssemblyStateCreateInfo;

    fn default_ci() -> Self::CIType {

        vk::PipelineInputAssemblyStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags : vk::PipelineInputAssemblyStateCreateFlags::empty(),
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: vk::FALSE,
        }
    }
}

impl InputAssemblySCI {

    pub fn new() -> InputAssemblySCI {

        InputAssemblySCI {
            sci: InputAssemblySCI::default_ci(),
        }
    }

    pub fn value(&self) -> vk::PipelineInputAssemblyStateCreateInfo {
        self.sci.clone()
    }

    pub fn topology(mut self, topology: vk::PrimitiveTopology) -> InputAssemblySCI {
        self.sci.topology = topology; self
    }

    pub fn primitive_restart(mut self, is_enable: bool) -> InputAssemblySCI {
        self.sci.primitive_restart_enable = if is_enable { vk::TRUE } else { vk::FALSE }; self
    }

    pub fn flags(mut self, flags: vk::PipelineInputAssemblyStateCreateFlags) -> InputAssemblySCI {
        self.sci.flags = flags; self
    }
}

impl From<InputAssemblySCI> for vk::PipelineInputAssemblyStateCreateInfo {

    fn from(value: InputAssemblySCI) -> vk::PipelineInputAssemblyStateCreateInfo {
        value.sci
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::PipelineRasterizationStateCreateInfo.
#[derive(Debug, Clone)]
pub struct RasterizationSCI {
    sci: vk::PipelineRasterizationStateCreateInfo,
}

impl VulkanCI for RasterizationSCI {
    type CIType = vk::PipelineRasterizationStateCreateInfo;

    fn default_ci() -> Self::CIType {

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

impl RasterizationSCI {

    pub fn new() -> RasterizationSCI {

        RasterizationSCI {
            sci: RasterizationSCI::default_ci(),
        }
    }

    pub fn value(&self) -> vk::PipelineRasterizationStateCreateInfo {
        self.sci.clone()
    }

    pub fn depth_clamp(mut self, is_enable: bool, bias: vkfloat) -> RasterizationSCI {
        self.sci.depth_clamp_enable = if is_enable { vk::TRUE } else { vk::FALSE };
        self.sci.depth_bias_clamp = bias; self
    }

    pub fn rasterizer_discard(mut self, is_enable: bool) -> RasterizationSCI {
        self.sci.rasterizer_discard_enable = if is_enable { vk::TRUE } else { vk::FALSE }; self
    }

    pub fn cull_face(mut self, mode: vk::CullModeFlags, front_face: vk::FrontFace) -> RasterizationSCI {
        self.sci.cull_mode = mode;
        self.sci.front_face = front_face; self
    }

    pub fn polygon(mut self, mode: vk::PolygonMode) -> RasterizationSCI {
        self.sci.polygon_mode = mode; self
    }

    pub fn depth_bias(mut self, is_enable: bool, constant_factor: vkfloat, slope_factor: vkfloat) -> RasterizationSCI {
        self.sci.depth_bias_enable = if is_enable { vk::TRUE } else { vk::FALSE };
        self.sci.depth_bias_constant_factor = constant_factor;
        self.sci.depth_bias_slope_factor = slope_factor; self
    }

    pub fn line_width(mut self, width: vkfloat) -> RasterizationSCI {
        self.sci.line_width = width; self
    }
}

impl From<RasterizationSCI> for vk::PipelineRasterizationStateCreateInfo {

    fn from(value: RasterizationSCI) -> vk::PipelineRasterizationStateCreateInfo {
        value.sci
    }
}
// ----------------------------------------------------------------------------------------------


// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::PipelineColorBlendStateCreateInfo.
#[derive(Debug, Clone)]
pub struct ColorBlendSCI {
    sci: vk::PipelineColorBlendStateCreateInfo,
    attachments: Vec<vk::PipelineColorBlendAttachmentState>,
}

impl VulkanCI for ColorBlendSCI {
    type CIType = vk::PipelineColorBlendStateCreateInfo;

    fn default_ci() -> Self::CIType {

        vk::PipelineColorBlendStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags : vk::PipelineColorBlendStateCreateFlags::empty(),
            logic_op_enable: vk::FALSE,
            logic_op       : vk::LogicOp::COPY,
            attachment_count: 0,
            p_attachments   : ptr::null(),
            blend_constants : [0.0; 4]
        }
    }
}

impl ColorBlendSCI {

    pub fn new() -> ColorBlendSCI {

        ColorBlendSCI {
            sci: ColorBlendSCI::default_ci(),
            attachments: Vec::new(),
        }
    }

    pub fn value(&self) -> vk::PipelineColorBlendStateCreateInfo {

        vk::PipelineColorBlendStateCreateInfo {
            attachment_count: self.attachments.len() as _,
            p_attachments   : self.attachments.as_ptr(),
            ..ColorBlendSCI::default_ci()
        }
    }

    pub fn add_attachment(mut self, attachment: vk::PipelineColorBlendAttachmentState) -> ColorBlendSCI {
        self.attachments.push(attachment); self
    }

    pub fn logic_op(mut self, is_enable: bool, op: vk::LogicOp) -> ColorBlendSCI {
        self.sci.logic_op_enable = if is_enable { vk::TRUE } else { vk::FALSE };
        self.sci.logic_op = op; self
    }

    pub fn blend_constants(mut self, constants: [vkfloat; 4]) -> ColorBlendSCI {
        self.sci.blend_constants = constants; self
    }

    pub fn flags(mut self, flags: vk::PipelineColorBlendStateCreateFlags) -> ColorBlendSCI {
        self.sci.flags = flags; self
    }
}

/// Wrapper class for vk::PipelineColorBlendStateCreateInfo.
#[derive(Debug, Clone)]
pub struct BlendAttachmentSCI {
    sci: vk::PipelineColorBlendAttachmentState,
}

impl VulkanCI for BlendAttachmentSCI {
    type CIType = vk::PipelineColorBlendAttachmentState;

    fn default_ci() -> Self::CIType {

        vk::PipelineColorBlendAttachmentState {
            blend_enable: vk::FALSE,
            src_color_blend_factor: vk::BlendFactor::ONE,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B | vk::ColorComponentFlags::A,
        }
    }
}

impl BlendAttachmentSCI {

    pub fn new() -> BlendAttachmentSCI {

        BlendAttachmentSCI {
            sci: BlendAttachmentSCI::default_ci(),
        }
    }

    pub fn value(self) -> vk::PipelineColorBlendAttachmentState {
        self.sci
    }

    pub fn blend_enable(mut self, is_enable: bool) -> BlendAttachmentSCI {
        self.sci.blend_enable = if is_enable { vk::TRUE } else { vk::FALSE }; self
    }

    pub fn color(mut self, op: vk::BlendOp, src_factor: vk::BlendFactor, dst_factor: vk::BlendFactor) -> BlendAttachmentSCI {
        self.sci.src_color_blend_factor = src_factor;
        self.sci.dst_color_blend_factor = dst_factor;
        self.sci.color_blend_op = op; self
    }

    pub fn alpha(mut self, op: vk::BlendOp, src_factor: vk::BlendFactor, dst_factor: vk::BlendFactor) -> BlendAttachmentSCI {
        self.sci.src_alpha_blend_factor = src_factor;
        self.sci.dst_alpha_blend_factor = dst_factor;
        self.sci.alpha_blend_op = op; self
    }

    pub fn color_write_mask(mut self, mask: vk::ColorComponentFlags) -> BlendAttachmentSCI {
        self.sci.color_write_mask = mask; self
    }
}

impl From<BlendAttachmentSCI> for vk::PipelineColorBlendAttachmentState {

    fn from(value: BlendAttachmentSCI) -> vk::PipelineColorBlendAttachmentState {
        value.sci
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::PipelineViewportStateCreateInfo.
#[derive(Debug, Clone)]
pub struct ViewportSCI {
    sci: vk::PipelineViewportStateCreateInfo,
    viewports: Vec<vk::Viewport>,
    scissors : Vec<vk::Rect2D>,
}

impl VulkanCI for ViewportSCI {
    type CIType = vk::PipelineViewportStateCreateInfo;

    fn default_ci() -> Self::CIType {

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

impl ViewportSCI {

    pub fn new() -> ViewportSCI {

        ViewportSCI {
            sci: ViewportSCI::default_ci(),
            viewports: Vec::new(),
            scissors : Vec::new(),
        }
    }

    pub fn value(&self) -> vk::PipelineViewportStateCreateInfo {

        vk::PipelineViewportStateCreateInfo {
            viewport_count: self.viewports.len() as _,
            p_viewports   : self.viewports.as_ptr(),
            scissor_count : self.scissors.len() as _,
            p_scissors    : self.scissors.as_ptr(),
            ..self.sci
        }
    }

    pub fn add_viewport(mut self, viewport: vk::Viewport) -> ViewportSCI {
        self.viewports.push(viewport); self
    }

    pub fn add_scissor(mut self, scissor: vk::Rect2D) -> ViewportSCI {
        self.scissors.push(scissor); self
    }

    pub fn flags(mut self, flags: vk::PipelineViewportStateCreateFlags) -> ViewportSCI {
        self.sci.flags = flags; self
    }
}
// ----------------------------------------------------------------------------------------------


// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::PipelineDepthStencilStateCreateInfo.
#[derive(Debug, Clone)]
pub struct DepthStencilSCI {
    sci: vk::PipelineDepthStencilStateCreateInfo,
}

impl VulkanCI for DepthStencilSCI {
    type CIType = vk::PipelineDepthStencilStateCreateInfo;

    fn default_ci() -> Self::CIType {

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
            depth_test_enable        : vk::TRUE,
            depth_write_enable       : vk::TRUE,
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

impl DepthStencilSCI {

    pub fn new() -> DepthStencilSCI {

        DepthStencilSCI {
            sci: DepthStencilSCI::default_ci(),
        }
    }

    pub fn value(&self) -> vk::PipelineDepthStencilStateCreateInfo {
        self.sci.clone()
    }

    pub fn depth_test(mut self, is_enable_test: bool, is_enable_write: bool, compare_op: vk::CompareOp) -> DepthStencilSCI {
        self.sci.depth_test_enable = if is_enable_test { vk::TRUE } else { vk::FALSE };
        self.sci.depth_write_enable = if is_enable_write { vk::TRUE } else { vk::FALSE };
        self.sci.depth_compare_op = compare_op; self
    }

    pub fn depth_bounds(mut self, is_enable: bool, min: vkfloat, max: vkfloat) -> DepthStencilSCI {
        self.sci.depth_bounds_test_enable = if is_enable { vk::TRUE } else { vk::FALSE };
        self.sci.min_depth_bounds = min;
        self.sci.max_depth_bounds = max; self
    }

    pub fn stencil(mut self, is_enable: bool, front: vk::StencilOpState, back: vk::StencilOpState) -> DepthStencilSCI {
        self.sci.stencil_test_enable =  if is_enable { vk::TRUE } else { vk::FALSE };
        self.sci.front = front;
        self.sci.back  = back; self
    }

    pub fn flags(mut self, flags: vk::PipelineDepthStencilStateCreateFlags) -> DepthStencilSCI {
        self.sci.flags = flags; self
    }
}

impl From<DepthStencilSCI> for vk::PipelineDepthStencilStateCreateInfo {

    fn from(value: DepthStencilSCI) -> vk::PipelineDepthStencilStateCreateInfo {
        value.sci
    }
}
// ----------------------------------------------------------------------------------------------



// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::PipelineMultisampleStateCreateInfo.
#[derive(Debug, Clone)]
pub struct MultisampleSCI {
    sci: vk::PipelineMultisampleStateCreateInfo,
    sample_mask: Option<vk::SampleMask>,
}

impl VulkanCI for MultisampleSCI {
    type CIType = vk::PipelineMultisampleStateCreateInfo;

    fn default_ci() -> Self::CIType {

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

impl MultisampleSCI {

    pub fn new() -> MultisampleSCI {

        MultisampleSCI {
            sci: MultisampleSCI::default_ci(),
            sample_mask: None,
        }
    }

    pub fn value(&self) -> vk::PipelineMultisampleStateCreateInfo {

        vk::PipelineMultisampleStateCreateInfo {
            p_sample_mask: if let Some(ref sample_mask) = self.sample_mask { sample_mask } else { ptr::null() },
            ..self.sci
        }
    }

    pub fn sample_count(mut self, count: vk::SampleCountFlags) -> MultisampleSCI {
        self.sci.rasterization_samples = count; self
    }

    pub fn sample_shading(mut self, is_enable: bool, min: vkfloat) -> MultisampleSCI {
        self.sci.sample_shading_enable = if is_enable { vk::TRUE } else { vk::FALSE };
        self.sci.min_sample_shading = min; self
    }

    pub fn sample_mask(mut self, mask: vk::SampleMask) -> MultisampleSCI {
        self.sample_mask = Some(mask); self
    }

    pub fn alpha(mut self, is_enable_alpha2coverage: bool, is_enable_alpha2one: bool) -> MultisampleSCI {
        self.sci.alpha_to_coverage_enable = if is_enable_alpha2coverage { vk::TRUE } else { vk::FALSE };
        self.sci.alpha_to_one_enable = if is_enable_alpha2one { vk::TRUE } else { vk::FALSE }; self
    }

    pub fn flags(mut self, flags: vk::PipelineMultisampleStateCreateFlags) -> MultisampleSCI {
        self.sci.flags = flags; self
    }
}

impl From<MultisampleSCI> for vk::PipelineMultisampleStateCreateInfo {

    fn from(value: MultisampleSCI) -> vk::PipelineMultisampleStateCreateInfo {
        value.sci
    }
}
// ----------------------------------------------------------------------------------------------


// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::PipelineDynamicStateCreateInfo.
#[derive(Debug, Clone)]
pub struct DynamicSCI {
    sci: vk::PipelineDynamicStateCreateInfo,
    dynamics: Vec<vk::DynamicState>,
}

impl VulkanCI for DynamicSCI {
    type CIType = vk::PipelineDynamicStateCreateInfo;

    fn default_ci() -> Self::CIType {

        vk::PipelineDynamicStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags : vk::PipelineDynamicStateCreateFlags::empty(),
            dynamic_state_count: 0,
            p_dynamic_states   : ptr::null(),
        }
    }
}

impl DynamicSCI {

    pub fn new() -> DynamicSCI {

        DynamicSCI {
            sci: DynamicSCI::default_ci(),
            dynamics: Vec::new(),
        }
    }

    pub fn add_dynamic(mut self, state: vk::DynamicState) -> DynamicSCI {
        self.dynamics.push(state); self
    }

    pub fn value(&self) -> vk::PipelineDynamicStateCreateInfo {

        vk::PipelineDynamicStateCreateInfo {
            dynamic_state_count: self.dynamics.len() as _,
            p_dynamic_states   : self.dynamics.as_ptr(),
            ..self.sci
        }
    }
}
// ----------------------------------------------------------------------------------------------



use ash::vk;

use crate::ci::VulkanCI;
use crate::{vkfloat, vkuint};

use std::ptr;

// ----------------------------------------------------------------------------------------------
/// Wrapper class for `vk::PipelineVertexInputStateCreateInfo`.
///
/// The default values are defined as follows:
/// ``` ignore
/// vk::PipelineVertexInputStateCreateInfo {
///     s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
///     p_next: ptr::null(),
///     flags : vk::PipelineVertexInputStateCreateFlags::empty(),
///     vertex_binding_description_count   : 0,
///     p_vertex_binding_descriptions      : ptr::null(),
///     vertex_attribute_description_count : 0,
///     p_vertex_attribute_descriptions    : ptr::null()
/// }
/// ```
///
/// See [VkPipelineVertexInputStateCreateInfo](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkPipelineVertexInputStateCreateInfo.html) for more detail.
///
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
            p_vertex_attribute_descriptions   : ptr::null()
        }
    }
}

impl AsRef<vk::PipelineVertexInputStateCreateInfo> for VertexInputSCI {

    fn as_ref(&self) -> &vk::PipelineVertexInputStateCreateInfo {
        &self.inner
    }
}

impl Default for VertexInputSCI {

    fn default() -> VertexInputSCI {
        VertexInputSCI {
            inner: VertexInputSCI::default_ci(),
            bindings  : Vec::new(),
            attributes: Vec::new(),
        }
    }
}

impl VertexInputSCI {

    /// Initialize `vk::PipelineVertexInputStateCreateInfo` with default value.
    #[inline(always)]
    pub fn new() -> VertexInputSCI {
        Default::default()
    }

    /// Add a vertex binding to `vk::PipelineVertexInputStateCreateInfo`.
    ///
    /// `binding` is the description of this binding.
    #[inline(always)]
    pub fn add_binding(mut self, binding: vk::VertexInputBindingDescription) -> VertexInputSCI {

        self.bindings.push(binding);
        self.inner.vertex_binding_description_count = self.bindings.len() as _;
        self.inner.p_vertex_binding_descriptions    = self.bindings.as_ptr(); self
    }

    /// Add a vertex attribute to `vk::PipelineVertexInputStateCreateInfo`.
    ///
    /// `attribute` is the vertex attribute used in a specific vertex binding.
    #[inline(always)]
    pub fn add_attribute(mut self, attribute: vk::VertexInputAttributeDescription) -> VertexInputSCI {

        self.attributes.push(attribute);
        self.inner.vertex_attribute_description_count = self.attributes.len() as _;
        self.inner.p_vertex_attribute_descriptions    = self.attributes.as_ptr(); self
    }

    /// Set the `flags` member for `vk::PipelineVertexInputStateCreateFlags`.
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
/// Wrapper class for `vk::PipelineInputAssemblyStateCreateInfo`.
///
/// The default values are defined as follows:
/// ``` ignore
/// vk::PipelineInputAssemblyStateCreateInfo {
///     s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
///     p_next: ptr::null(),
///     flags : vk::PipelineInputAssemblyStateCreateFlags::empty(),
///     topology: vk::PrimitiveTopology::TRIANGLE_LIST,
///     primitive_restart_enable: vk::FALSE,
/// }
/// ```
///
/// See [VKPipelineInputAssemblyStateCreateInfo](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VKPipelineInputAssemblyStateCreateInfo.html) for more detail.
///
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

impl AsRef<vk::PipelineInputAssemblyStateCreateInfo> for InputAssemblySCI {

    fn as_ref(&self) -> &vk::PipelineInputAssemblyStateCreateInfo {
        &self.inner
    }
}

impl Default for InputAssemblySCI {

    fn default() -> InputAssemblySCI {
        InputAssemblySCI {
            inner: InputAssemblySCI::default_ci(),
        }
    }
}

impl InputAssemblySCI {

    /// Initialize `vk::PipelineInputAssemblyStateCreateInfo` with default value.
    #[inline(always)]
    pub fn new() -> InputAssemblySCI {
        Default::default()
    }

    /// Set the `topology` member for `vk::PipelineInputAssemblyStateCreateInfo`.
    ///
    /// `topology` specifies the primitive topology.
    #[inline(always)]
    pub fn topology(mut self, topology: vk::PrimitiveTopology) -> InputAssemblySCI {
        self.inner.topology = topology; self
    }

    /// Set the `primitive_restart_enable` member for `vk::PipelineInputAssemblyStateCreateInfo`.
    ///
    /// `is_enable` controls whether a special vertex index value is treated as restarting the assembly of primitives. Disable by default.
    #[inline(always)]
    pub fn primitive_restart(mut self, is_enable: bool) -> InputAssemblySCI {
        self.inner.primitive_restart_enable = if is_enable { vk::TRUE } else { vk::FALSE }; self
    }

    /// Set the `flags` member for `vk::PipelineInputAssemblyStateCreateInfo`.
    #[inline(always)]
    pub fn flags(mut self, flags: vk::PipelineInputAssemblyStateCreateFlags) -> InputAssemblySCI {
        self.inner.flags = flags; self
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
/// Wrapper class for `vk::PipelineRasterizationStateCreateInfo`.
///
/// The default values are defined as follows:
/// ``` ignore
/// vk::PipelineRasterizationStateCreateInfo {
///     s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
///     p_next: ptr::null(),
///     flags : vk::PipelineRasterizationStateCreateFlags::empty(),
///     depth_clamp_enable         : vk::FALSE,
///     rasterizer_discard_enable  : vk::FALSE,
///     polygon_mode               : vk::PolygonMode::FILL,
///     cull_mode                  : vk::CullModeFlags::NONE,
///     front_face                 : vk::FrontFace::COUNTER_CLOCKWISE,
///     depth_bias_enable          : vk::FALSE,
///     depth_bias_constant_factor : 0.0,
///     depth_bias_clamp           : 0.0,
///     depth_bias_slope_factor    : 0.0,
///     line_width                 : 1.0,
/// }
/// ```
///
/// See [VkPipelineRasterizationStateCreateInfo](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkPipelineRasterizationStateCreateInfo.html) for more detail.
///
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

impl AsRef<vk::PipelineRasterizationStateCreateInfo> for RasterizationSCI {

    fn as_ref(&self) -> &vk::PipelineRasterizationStateCreateInfo {
        &self.inner
    }
}

impl Default for RasterizationSCI {

    fn default() -> RasterizationSCI {
        RasterizationSCI {
            inner: RasterizationSCI::default_ci(),
        }
    }
}

impl RasterizationSCI {

    /// Initialize `vk::PipelineRasterizationStateCreateInfo` with default value.
    #[inline(always)]
    pub fn new() -> RasterizationSCI {
        Default::default()
    }

    /// Set the `depth_clamp_enable` and `depth_bias_clamp` members for `vk::PipelineRasterizationStateCreateInfo`.
    ///
    /// `depth_clamp_enable` controls whether to clamp the fragment’s depth values in Depth Test.
    ///
    /// `bias` is the maximum (or minimum) depth bias of a fragment.
    #[inline(always)]
    pub fn depth_clamp(mut self, is_enable: bool, bias: vkfloat) -> RasterizationSCI {
        self.inner.depth_clamp_enable = if is_enable { vk::TRUE } else { vk::FALSE };
        self.inner.depth_bias_clamp = bias; self
    }

    /// Set the `rasterizer_discard_enable` member for `vk::PipelineRasterizationStateCreateInfo`.
    ///
    /// `is_enable` controls whether primitives are discarded immediately before the rasterization stage.
    #[inline(always)]
    pub fn rasterizer_discard(mut self, is_enable: bool) -> RasterizationSCI {
        self.inner.rasterizer_discard_enable = if is_enable { vk::TRUE } else { vk::FALSE }; self
    }

    /// Set the `cull_mode` and `front_face` members for `vk::PipelineRasterizationStateCreateInfo`.
    ///
    /// `mode` specifies the triangle facing direction used in primitive culling.
    ///
    /// `front_face` specifies the front-facing triangle orientation to be used for culling.
    #[inline(always)]
    pub fn cull_face(mut self, mode: vk::CullModeFlags, front_face: vk::FrontFace) -> RasterizationSCI {
        self.inner.cull_mode = mode;
        self.inner.front_face = front_face; self
    }

    /// Set the `polygon_mode` member for `vk::PipelineRasterizationStateCreateInfo`.
    ///
    /// `mode` specifies the triangle rendering mode.
    #[inline(always)]
    pub fn polygon(mut self, mode: vk::PolygonMode) -> RasterizationSCI {
        self.inner.polygon_mode = mode; self
    }

    /// Set the `depth_bias_enable`, `depth_bias_constant_factor` and `depth_bias_slope_factor` members for `vk::PipelineRasterizationStateCreateInfo`.
    ///
    /// `is_enable` controls whether to bias fragment depth values.
    ///
    /// `constant_factor` is a scalar factor controlling the constant depth value added to each fragment.
    ///
    /// `slope_factor` is a scalar factor applied to a fragment’s slope in depth bias calculations.
    #[inline(always)]
    pub fn depth_bias(mut self, is_enable: bool, constant_factor: vkfloat, slope_factor: vkfloat) -> RasterizationSCI {
        self.inner.depth_bias_enable = if is_enable { vk::TRUE } else { vk::FALSE };
        self.inner.depth_bias_constant_factor = constant_factor;
        self.inner.depth_bias_slope_factor = slope_factor; self
    }

    /// Set the `line_width` member for `PipelineRasterizationStateCreateInfo`.
    ///
    /// `width` is the width of rasterized line segments.
    #[inline(always)]
    pub fn line_width(mut self, width: vkfloat) -> RasterizationSCI {
        self.inner.line_width = width; self
    }

    /// Set the `flags` member for `vk::PipelineRasterizationStateCreateInfo`.
    #[inline(always)]
    pub fn flags(mut self, flags: vk::PipelineRasterizationStateCreateFlags) -> RasterizationSCI {
        self.inner.flags = flags; self
    }
}
// ----------------------------------------------------------------------------------------------


// ----------------------------------------------------------------------------------------------
/// Wrapper class for `vk::PipelineColorBlendStateCreateInfo`.
///
/// The default values are defined as follows:
/// ```ignore
/// vk::PipelineColorBlendStateCreateInfo {
///     s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
///     p_next: ptr::null(),
///     flags : vk::PipelineColorBlendStateCreateFlags::empty(),
///     logic_op_enable: vk::FALSE,
///     logic_op       : vk::LogicOp::COPY,
///     attachment_count: 0,
///     p_attachments   : ptr::null(),
///     blend_constants : [0.0; 4],
/// }
/// ```
///
/// See [VkPipelineColorBlendStateCreateInfo](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkPipelineColorBlendStateCreateInfo.html) for more detail.
///
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

impl AsRef<vk::PipelineColorBlendStateCreateInfo> for ColorBlendSCI {

    fn as_ref(&self) -> &vk::PipelineColorBlendStateCreateInfo {
        &self.inner
    }
}

impl Default for ColorBlendSCI {

    fn default() -> ColorBlendSCI {
        ColorBlendSCI {
            inner: ColorBlendSCI::default_ci(),
            attachments: Vec::new(),
        }
    }
}

impl ColorBlendSCI {

    /// Initialize `vk::PipelineColorBlendStateCreateInfo` with default value.
    #[inline(always)]
    pub fn new() -> ColorBlendSCI {
        Default::default()
    }

    /// Add a blend attachment to `vk::PipelineColorBlendStateCreateInfo`.
    ///
    /// `attachment` specifies per-target blending state for each individual color attachment.
    /// This total count of blend attachment must equal the count of color attachments for the subpass in which this pipeline is used.
    #[inline(always)]
    pub fn add_attachment(mut self, attachment: BlendAttachmentSCI) -> ColorBlendSCI {

        self.attachments.push(attachment.into());
        self.inner.attachment_count = self.attachments.len() as _;
        self.inner.p_attachments    = self.attachments.as_ptr(); self
    }

    /// Set the `logic_op_enable` and `logic_op` members for `vk::PipelineColorBlendStateCreateInfo`.
    ///
    /// `is_enable` specifies whether to apply Logical Operations. Default is false.
    ///
    /// `op` specifies which logical operation to apply if `is_enable is true`.
    #[inline(always)]
    pub fn logic_op(mut self, is_enable: bool, op: vk::LogicOp) -> ColorBlendSCI {
        self.inner.logic_op_enable = if is_enable { vk::TRUE } else { vk::FALSE };
        self.inner.logic_op = op; self
    }

    /// Set the `constants` member for `vk::PipelineColorBlendStateCreateInfo`.
    ///
    /// `constants` is the R, G, B, and A components of the blend constant used in blending.
    #[inline(always)]
    pub fn blend_constants(mut self, constants: [vkfloat; 4]) -> ColorBlendSCI {
        self.inner.blend_constants = constants; self
    }

    /// Set the `flags` member for `vk::PipelineColorBlendStateCreateInfo`.
    #[inline(always)]
    pub fn flags(mut self, flags: vk::PipelineColorBlendStateCreateFlags) -> ColorBlendSCI {
        self.inner.flags = flags; self
    }
}
// ----------------------------------------------------------------------------------------------



// ----------------------------------------------------------------------------------------------
/// Wrapper class for `vk::PipelineColorBlendAttachmentState`.
///
/// The default values are defined as follows:
/// ```ignore
/// vk::PipelineColorBlendAttachmentState {
///     blend_enable: vk::FALSE,
///     src_color_blend_factor: vk::BlendFactor::ONE,
///     dst_color_blend_factor: vk::BlendFactor::ZERO,
///     color_blend_op: vk::BlendOp::ADD,
///     src_alpha_blend_factor: vk::BlendFactor::ONE,
///     dst_alpha_blend_factor: vk::BlendFactor::ZERO,
///     alpha_blend_op: vk::BlendOp::ADD,
///     color_write_mask:vk::ColorComponentFlags::R
///     | vk::ColorComponentFlags::G
///     | vk::ColorComponentFlags::B
///     | vk::ColorComponentFlags::A,
/// }
/// ```
///
/// See [VkPipelineColorBlendAttachmentState](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkPipelineColorBlendAttachmentState.html) for more detail.
///
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

impl AsRef<vk::PipelineColorBlendAttachmentState> for BlendAttachmentSCI {

    fn as_ref(&self) -> &vk::PipelineColorBlendAttachmentState {
        &self.inner
    }
}

impl Default for BlendAttachmentSCI {

    fn default() -> BlendAttachmentSCI {
        BlendAttachmentSCI {
            inner: BlendAttachmentSCI::default_ci(),
        }
    }
}

impl BlendAttachmentSCI {

    /// Initialize `vk::PipelineColorBlendAttachmentState` with default value.
    #[inline(always)]
    pub fn new() -> BlendAttachmentSCI {
        Default::default()
    }

    /// Set the `blend_enable` member for `vk::PipelineColorBlendAttachmentState`.
    ///
    /// `is_enable` controls whether blending is enabled for the corresponding color attachment.
    #[inline(always)]
    pub fn blend_enable(mut self, is_enable: bool) -> BlendAttachmentSCI {
        self.inner.blend_enable = if is_enable { vk::TRUE } else { vk::FALSE }; self
    }

    /// Set the `color_blend_op`, `src_color_blend_factor` and `dst_color_blend_factor` members for `vk::PipelineColorBlendAttachmentState`.
    ///
    /// `op` specifies which blend operation is use to calculate the RGB values to write to the color attachment.
    ///
    /// `src_factor` specifies the source rgb factor in blending.
    ///
    /// `dst_factor` specifies the destination rgb factor in blending.
    #[inline(always)]
    pub fn color(mut self, op: vk::BlendOp, src_factor: vk::BlendFactor, dst_factor: vk::BlendFactor) -> BlendAttachmentSCI {
        self.inner.src_color_blend_factor = src_factor;
        self.inner.dst_color_blend_factor = dst_factor;
        self.inner.color_blend_op = op; self
    }

    /// Set the `alpha_blend_op`, `src_alpha_blend_factor` and `dst_alpha_blend_factor` members for `vk::PipelineColorBlendAttachmentState`.
    ///
    /// `op` specifies which blend operation is use to calculate the alpha values to write to the color attachment.
    ///
    /// `src_factor` specifies the source alpha factor in blending.
    ///
    /// `dst_factor` specifies the destination alpha factor in blending.
    #[inline(always)]
    pub fn alpha(mut self, op: vk::BlendOp, src_factor: vk::BlendFactor, dst_factor: vk::BlendFactor) -> BlendAttachmentSCI {
        self.inner.src_alpha_blend_factor = src_factor;
        self.inner.dst_alpha_blend_factor = dst_factor;
        self.inner.alpha_blend_op = op; self
    }

    /// Set the `color_write_mask` member for `vk::PipelineColorBlendAttachmentState`.
    ///
    /// `mask` specifies the R, G, B, and/or A components are enabled for writing.
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
/// Wrapper class for `vk::PipelineViewportStateCreateInfo`.
///
/// The default values are defined as follows:
/// ```ignore
/// vk::PipelineViewportStateCreateInfo {
///     s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
///     p_next: ptr::null(),
///     flags : vk::PipelineViewportStateCreateFlags::empty(),
///     viewport_count : 0,
///     p_viewports    : ptr::null(),
///     scissor_count  : 0,
///     p_scissors     : ptr::null(),
/// }
/// ```
///
/// See [VkPipelineViewportStateCreateInfo](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkPipelineViewportStateCreateInfo.html) for more detail.
///
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

impl AsRef<vk::PipelineViewportStateCreateInfo> for ViewportSCI {

    fn as_ref(&self) -> &vk::PipelineViewportStateCreateInfo {
        &self.inner
    }
}

impl Default for ViewportSCI {

    fn default() -> ViewportSCI {
        ViewportSCI {
            inner: ViewportSCI::default_ci(),
            viewports: Vec::new(),
            scissors : Vec::new(),
        }
    }
}

impl ViewportSCI {

    /// Initialize `vk::PipelineViewportStateCreateInfo` with default value.
    #[inline(always)]
    pub fn new() -> ViewportSCI {
        Default::default()
    }

    /// Set the `viewport_count` member for `vk::PipelineViewportStateCreateInfo`.
    ///
    /// `count` is the count of dynamic viewports.
    /// Use this method for pipeline with dynamic viewports.
    #[inline]
    pub fn with_dynamic_viewport_count(mut self, count: vkuint) -> ViewportSCI {

        self.viewports.clear();
        self.inner.viewport_count = count;
        self.inner.p_viewports    = ptr::null(); self
    }

    /// Set the `scissor_count` member for `vk::PipelineViewportStateCreateInfo`.
    ///
    /// `count` is the count of dynamic scissors.
    /// Use this method for pipeline with dynamic scissors.
    #[inline]
    pub fn with_dynamic_scissor_count(mut self, count: vkuint) -> ViewportSCI {

        self.scissors.clear();
        self.inner.scissor_count = count;
        self.inner.p_scissors    = ptr::null(); self
    }

    /// Add viewport to `vk::PipelineViewportStateCreateInfo`.
    ///
    /// `viewport` defines the viewport transforms.
    /// Use this method for pipeline with fixed viewports.
    #[inline(always)]
    pub fn add_viewport(mut self, viewport: vk::Viewport) -> ViewportSCI {

        self.viewports.push(viewport);
        self.inner.viewport_count = self.viewports.len() as _;
        self.inner.p_viewports    = self.viewports.as_ptr(); self
    }

    /// Add scissor to `vk::PipelineViewportStateCreateInfo`.
    ///
    /// `scissor` defines the rectangular bounds of the scissor for the corresponding viewport.
    /// Use this method for pipeline with fixed scissors.
    #[inline(always)]
    pub fn add_scissor(mut self, scissor: vk::Rect2D) -> ViewportSCI {

        self.scissors.push(scissor);
        self.inner.scissor_count = self.scissors.len() as _;
        self.inner.p_scissors    = self.scissors.as_ptr(); self
    }

    /// Set the `flags` member for `vk::PipelineViewportStateCreateInfo`.
    #[inline(always)]
    pub fn flags(mut self, flags: vk::PipelineViewportStateCreateFlags) -> ViewportSCI {
        self.inner.flags = flags; self
    }
}
// ----------------------------------------------------------------------------------------------


// ----------------------------------------------------------------------------------------------
/// Wrapper class for `vk::PipelineDepthStencilStateCreateInfo`.
///
/// The default values are defined as follows:
/// ```ignore
/// vk::PipelineDepthStencilStateCreateInfo {
///    s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
///    p_next: ptr::null(),
///    flags : vk::PipelineDepthStencilStateCreateFlags::empty(),
///    depth_test_enable        : vk::FALSE,
///    depth_write_enable       : vk::FALSE,
///    depth_compare_op         : vk::CompareOp::LESS_OR_EQUAL,
///    depth_bounds_test_enable : vk::FALSE,
///    stencil_test_enable      : vk::FALSE,
///    front: vk::StencilOpState {
///        fail_op: vk::StencilOp::KEEP,
///        pass_op: vk::StencilOp::KEEP,
///        depth_fail_op: vk::StencilOp::KEEP,
///        compare_op   : vk::CompareOp::ALWAYS,
///        compare_mask : 0,
///        write_mask   : 0,
///        reference    : 0,
///    },
///    back: vk::StencilOpState {
///        fail_op: vk::StencilOp::KEEP,
///        pass_op: vk::StencilOp::KEEP,
///        depth_fail_op: vk::StencilOp::KEEP,
///        compare_op   : vk::CompareOp::ALWAYS,
///        compare_mask : 0,
///        write_mask   : 0,
///        reference    : 0,
///    },
///    min_depth_bounds: 0.0,
///    max_depth_bounds: 1.0,
/// }
/// ```
///
/// See [VkPipelineDepthStencilStateCreateInfo](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkPipelineDepthStencilStateCreateInfo.html) for more detail.
///
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

impl AsRef<vk::PipelineDepthStencilStateCreateInfo> for DepthStencilSCI {

    fn as_ref(&self) -> &vk::PipelineDepthStencilStateCreateInfo {
        &self.inner
    }
}

impl Default for DepthStencilSCI {

    fn default() -> DepthStencilSCI {
        DepthStencilSCI {
            inner: DepthStencilSCI::default_ci(),
        }
    }
}

impl DepthStencilSCI {

    /// Initialize `vk::PipelineDepthStencilStateCreateInfo` with default value.
    #[inline(always)]
    pub fn new() -> DepthStencilSCI {
        Default::default()
    }

    /// Set the `depth_test_enable`, `depth_write_enable` and `depth_compare_op` members for `vk::PipelineDepthStencilStateCreateInfo`.
    ///
    /// `is_enable_test` controls whether depth testing is enabled.
    ///
    /// `is_enable_write` controls whether depth writes are enabled.
    ///
    /// `compare_op` specifies the comparison operator used in the depth test.
    #[inline(always)]
    pub fn depth_test(mut self, is_enable_test: bool, is_enable_write: bool, compare_op: vk::CompareOp) -> DepthStencilSCI {
        self.inner.depth_test_enable  = if is_enable_test  { vk::TRUE } else { vk::FALSE };
        self.inner.depth_write_enable = if is_enable_write { vk::TRUE } else { vk::FALSE };
        self.inner.depth_compare_op   = compare_op; self
    }

    /// Set the `depth_bounds_test_enable`, `min_depth_bounds` and `max_depth_bounds` members for `vk::PipelineDepthStencilStateCreateInfo`.
    ///
    /// `is_enable` controls whether depth testing is enabled.
    ///
    /// `min` and `max` define the range of values used in the depth bounds test.
    #[inline(always)]
    pub fn depth_bounds(mut self, is_enable: bool, min: vkfloat, max: vkfloat) -> DepthStencilSCI {
        self.inner.depth_bounds_test_enable = if is_enable { vk::TRUE } else { vk::FALSE };
        self.inner.min_depth_bounds = min;
        self.inner.max_depth_bounds = max; self
    }

    /// Set the `stencil_test_enable`, `front` and `back` members for `vk::PipelineDepthStencilStateCreateInfo`.
    ///
    /// `is_enable_test` controls whether stencil testing is enabled.
    ///
    /// `front` and `back` control the parameters of the stencil test.
    #[inline(always)]
    pub fn stencil(mut self, is_enable: bool, front: vk::StencilOpState, back: vk::StencilOpState) -> DepthStencilSCI {
        self.inner.stencil_test_enable =  if is_enable { vk::TRUE } else { vk::FALSE };
        self.inner.front = front;
        self.inner.back  = back; self
    }

    /// Set the `flags` member for `vk::PipelineDepthStencilStateCreateInfo`.
    #[inline(always)]
    pub fn flags(mut self, flags: vk::PipelineDepthStencilStateCreateFlags) -> DepthStencilSCI {
        self.inner.flags = flags; self
    }
}
// ----------------------------------------------------------------------------------------------



// ----------------------------------------------------------------------------------------------
/// Wrapper class for `vk::PipelineMultisampleStateCreateInfo`.
///
/// The default values are defined as follows:
/// ```ignore
/// vk::PipelineMultisampleStateCreateInfo {
///     s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
///     p_next: ptr::null(),
///     flags : vk::PipelineMultisampleStateCreateFlags::empty(),
///     rasterization_samples: vk::SampleCountFlags::TYPE_1,
///     sample_shading_enable: vk::FALSE,
///     min_sample_shading: 0.0,
///     p_sample_mask: ptr::null(),
///     alpha_to_coverage_enable: vk::FALSE,
///     alpha_to_one_enable     : vk::FALSE,
/// }
/// ```
///
/// See [VkPipelineMultisampleStateCreateInfo](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkPipelineMultisampleStateCreateInfo.html) for more detail.
///
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

impl AsRef<vk::PipelineMultisampleStateCreateInfo> for MultisampleSCI {

    fn as_ref(&self) -> &vk::PipelineMultisampleStateCreateInfo {
        &self.inner
    }
}

impl Default for MultisampleSCI {

    fn default() -> MultisampleSCI {
        MultisampleSCI {
            inner: MultisampleSCI::default_ci(),
            sample_mask: None,
        }
    }
}

impl MultisampleSCI {

    /// Initialize `vk::PipelineMultisampleStateCreateInfo` with default value.
    #[inline(always)]
    pub fn new() -> MultisampleSCI {
        Default::default()
    }

    /// Set the `rasterization_samples` member for `vk::PipelineMultisampleStateCreateInfo`.
    ///
    /// `count` specifies the number of samples used in rasterization. Default is `vk::SampleCountFlags::TYPE_1`.
    #[inline(always)]
    pub fn sample_count(mut self, count: vk::SampleCountFlags) -> MultisampleSCI {
        self.inner.rasterization_samples = count; self
    }

    /// Set the `sample_shading_enable` and `min_sample_shading` member for `vk::PipelineMultisampleStateCreateInfo`.
    ///
    /// `is_enable` controls whether sample shading is enabled. Default is false.
    ///
    /// `min` specifies a minimum fraction of sample shading.
    #[inline(always)]
    pub fn sample_shading(mut self, is_enable: bool, min: vkfloat) -> MultisampleSCI {
        self.inner.sample_shading_enable = if is_enable { vk::TRUE } else { vk::FALSE };
        self.inner.min_sample_shading = min; self
    }

    /// Set the `sample_mask` member for `vk::PipelineMultisampleStateCreateInfo`.
    #[inline(always)]
    pub fn sample_mask(mut self, mask: vk::SampleMask) -> MultisampleSCI {
        self.inner.p_sample_mask = &mask;
        self.sample_mask = Some(mask); self
    }

    /// Set the `alpha_to_coverage_enable` and `alpha_to_one_enable` member for `vk::PipelineMultisampleStateCreateInfo`.
    #[inline(always)]
    pub fn alpha(mut self, is_enable_alpha2coverage: bool, is_enable_alpha2one: bool) -> MultisampleSCI {
        self.inner.alpha_to_coverage_enable = if is_enable_alpha2coverage { vk::TRUE } else { vk::FALSE };
        self.inner.alpha_to_one_enable = if is_enable_alpha2one { vk::TRUE } else { vk::FALSE }; self
    }

    /// Set the `flags` member for `vk::PipelineViewportStateCreateInfo`.
    #[inline(always)]
    pub fn flags(mut self, flags: vk::PipelineMultisampleStateCreateFlags) -> MultisampleSCI {
        self.inner.flags = flags; self
    }
}
// ----------------------------------------------------------------------------------------------


// ----------------------------------------------------------------------------------------------
/// Wrapper class for `vk::PipelineDynamicStateCreateInfo`.
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

impl AsRef<vk::PipelineDynamicStateCreateInfo> for DynamicSCI {

    fn as_ref(&self) -> &vk::PipelineDynamicStateCreateInfo {
        &self.inner
    }
}

impl Default for DynamicSCI {

    fn default() -> DynamicSCI {
        DynamicSCI {
            inner: DynamicSCI::default_ci(),
            dynamics: None,
        }
    }
}

impl DynamicSCI {

    /// Initialize `vk::PipelineDynamicStateCreateInfo` with default value.
    #[inline]
    pub fn new() -> DynamicSCI {
        Default::default()
    }

    /// Add dynamic state to `vk::PipelineDynamicStateCreateInfo`.
    ///
    /// `state` specifies which pieces of pipeline state will use the values from dynamic state commands.
    #[inline]
    pub fn add_dynamic(mut self, state: vk::DynamicState) -> DynamicSCI {

        let dynamics = self.dynamics.get_or_insert(Vec::new());
        dynamics.push(state);

        self.inner.dynamic_state_count = dynamics.len() as _;
        self.inner.p_dynamic_states    = dynamics.as_ptr(); self
    }

    /// Set the `flags` member for `vk::PipelineDynamicStateCreateInfo`.
    #[inline(always)]
    pub fn flags(mut self, flags: vk::PipelineDynamicStateCreateFlags) -> DynamicSCI {
        self.inner.flags = flags; self
    }
}
// ----------------------------------------------------------------------------------------------

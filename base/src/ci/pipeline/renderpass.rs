
use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::VkDevice;
use crate::context::VkObjectDiscardable;

use crate::ci::{VulkanCI, VkObjectBuildableCI};

use crate::error::{VkResult, VkError};
use crate::vkuint;

use std::ptr;

// ----------------------------------------------------------------------------------------------
/// Wrapper class for `vk::RenderPassBeginInfo`.
///
/// The default values are defined as follows:
/// ``` ignore
/// vk::RenderPassBeginInfo {
///     s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
///     p_next: ptr::null(),
///     render_area: vk::Rect2D {
///         extent: vk::Extent2D { width : 0, height: 0 },
///         offset: vk::Offset2D { x: 0, y: 0 },
///     },
///     clear_value_count: 0,
///     p_clear_values   : ptr::null(),
///     render_pass: vk::RenderPass::null(),
///     framebuffer: vk::Framebuffer::null(),
/// }
/// ```
///
/// See [VkRenderPassBeginInfo](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkRenderPassBeginInfo.html) for more detail.
///
#[derive(Clone)]
pub struct RenderPassBI {

    inner: vk::RenderPassBeginInfo,
    clears: Option<Vec<vk::ClearValue>>,
}

impl VulkanCI<vk::RenderPassBeginInfo> for RenderPassBI {

    fn default_ci() -> vk::RenderPassBeginInfo {

        vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            p_next: ptr::null(),
            render_area: vk::Rect2D {
                extent: vk::Extent2D { width : 0, height: 0 },
                offset: vk::Offset2D { x: 0, y: 0 },
            },
            clear_value_count: 0,
            p_clear_values   : ptr::null(),
            render_pass: vk::RenderPass::null(),
            framebuffer: vk::Framebuffer::null(),
        }
    }
}

impl AsRef<vk::RenderPassBeginInfo> for RenderPassBI {

    fn as_ref(&self) -> &vk::RenderPassBeginInfo {
        &self.inner
    }
}

impl RenderPassBI {

    /// Initialize `vk::RenderPassBeginInfo` with default value.
    ///
    /// `render_pass`is the handle of render pass for this operations.
    ///
    /// `framebuffer` is the framebuffer containing the attachments that are used with the render pass.
    pub fn new(render_pass: vk::RenderPass, framebuffer: vk::Framebuffer) -> RenderPassBI {

        RenderPassBI {
            inner: vk::RenderPassBeginInfo {
                render_pass, framebuffer,
                ..RenderPassBI::default_ci()
            },
            clears: None,
        }
    }

    /// Set the dimension of render area that is affected by this render pass.
    #[inline(always)]
    pub fn render_extent(mut self, area: vk::Extent2D) -> RenderPassBI {
        self.inner.render_area.extent = area; self
    }

    /// Set the offset of render area. Default is 0 for both x, y coordinates.
    #[inline(always)]
    pub fn render_area_offset(mut self, offset: vk::Offset2D) -> RenderPassBI {
        self.inner.render_area.offset = offset; self
    }

    /// Add clear value for attachment used in this render pass.
    #[inline]
    pub fn add_clear_value(mut self, value: vk::ClearValue) -> RenderPassBI {

        let clear_values = self.clears.get_or_insert(Vec::new());
        clear_values.push(value);

        self.inner.clear_value_count = clear_values.len() as _;
        self.inner.p_clear_values    = clear_values.as_ptr(); self
    }

    /// Set all the clear values for attachments used in this render pass.
    ///
    /// The order of clear values must match the corresponding attachment.
    #[inline]
    pub fn set_clear_values(mut self, values: Vec<vk::ClearValue>) -> RenderPassBI {

        self.inner.clear_value_count = values.len() as _;
        self.inner.p_clear_values    = values.as_ptr();

        self.clears.replace(values); self
    }
}
// ----------------------------------------------------------------------------------------------


// ----------------------------------------------------------------------------------------------
/// Wrapper class for `vk::RenderPassCreateInfo`.
///
/// The default values are defined as follows:
/// ``` ignore
/// vk::RenderPassCreateInfo {
///     s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
///     p_next: ptr::null(),
///     flags : vk::RenderPassCreateFlags::empty(),
///     attachment_count: 0,
///     p_attachments   : ptr::null(),
///     subpass_count   : 0,
///     p_subpasses     : ptr::null(),
///     dependency_count: 0,
///     p_dependencies  : ptr::null(),
/// }
/// ```
///
/// See [VkRenderPassCreateInfo](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkRenderPassCreateInfo.html) for more detail.
///
#[derive(Debug)]
pub struct RenderPassCI {

    inner: vk::RenderPassCreateInfo,
    attachments : Vec<vk::AttachmentDescription>,
    subpasses   : Vec<vk::SubpassDescription>,
    dependencies: Option<Vec<vk::SubpassDependency>>,

    subpass_cis: Vec<SubpassDescCI>,
}

impl VulkanCI<vk::RenderPassCreateInfo> for RenderPassCI {

    fn default_ci() -> vk::RenderPassCreateInfo {

        vk::RenderPassCreateInfo {
            s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
            p_next: ptr::null(),
            flags : vk::RenderPassCreateFlags::empty(),
            attachment_count: 0,
            p_attachments   : ptr::null(),
            subpass_count   : 0,
            p_subpasses     : ptr::null(),
            dependency_count: 0,
            p_dependencies  : ptr::null(),
        }
    }
}

impl AsRef<vk::RenderPassCreateInfo> for RenderPassCI {

    fn as_ref(&self) -> &vk::RenderPassCreateInfo {
        &self.inner
    }
}

impl VkObjectBuildableCI for RenderPassCI {
    type ObjectType = vk::RenderPass;

    /// Create `vk::RenderPass` object, and return its handle.
    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let render_pass = unsafe {
            device.logic.handle.create_render_pass(self.as_ref(), None)
                .map_err(|_| VkError::create("Render Pass"))?
        };
        Ok(render_pass)
    }
}

impl RenderPassCI {

    /// Initialize `vk::RenderPassCreateInfo` with default value.
    pub fn new() -> RenderPassCI {

        RenderPassCI {
            inner: RenderPassCI::default_ci(),
            attachments : Vec::new(),
            subpasses   : Vec::new(),
            dependencies: None,
            subpass_cis : Vec::new(),
        }
    }

    /// Add an attachment used by this render pass.
    #[inline]
    pub fn add_attachment(mut self, attachment: AttachmentDescCI) -> RenderPassCI {

        self.attachments.push(attachment.into());
        self.inner.attachment_count = self.attachments.len() as _;
        self.inner.p_attachments    = self.attachments.as_ptr(); self
    }

    /// Add a subpass to this render pass.
    #[inline]
    pub fn add_subpass(mut self, subpass: SubpassDescCI) -> RenderPassCI {

        if let Some(ref resolves) = subpass.resolves {
            assert_eq!(resolves.len() as vkuint, subpass.as_ref().color_attachment_count);
        }

        self.subpasses.push(subpass.inner.clone());
        self.subpass_cis.push(subpass);

        self.inner.subpass_count = self.subpasses.len() as _;
        self.inner.p_subpasses   = self.subpasses.as_ptr(); self
    }

    /// Add a subpass dependency between two subpass.
    #[inline]
    pub fn add_dependency(mut self, dependency: SubpassDependencyCI) -> RenderPassCI {

        let dependencies = self.dependencies.get_or_insert(Vec::new());
        dependencies.push(dependency.into());

        self.inner.dependency_count = dependencies.len() as _;
        self.inner.p_dependencies   = dependencies.as_ptr(); self
    }

    /// Set the `flags` member for `vk::RenderPassCreateInfo`.
    #[inline(always)]
    pub fn flags(mut self, flags: vk::RenderPassCreateFlags) -> RenderPassCI {
        self.inner.flags = flags; self
    }
}

impl VkObjectDiscardable for vk::RenderPass {

    fn discard_by(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_render_pass(self, None);
        }
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
/// Wrapper class for `vk::AttachmentDescription`.
///
/// The default values are defined as follows:
/// ``` ignore
/// vk::AttachmentDescription {
///     flags : vk::AttachmentDescriptionFlags::empty(),
///     format: vk::Format::UNDEFINED,
///     samples: vk::SampleCountFlags::TYPE_1,
///     load_op : vk::AttachmentLoadOp::DONT_CARE,
///     store_op: vk::AttachmentStoreOp::DONT_CARE,
///     stencil_load_op : vk::AttachmentLoadOp::DONT_CARE,
///     stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
///     initial_layout: vk::ImageLayout::UNDEFINED,
///     final_layout  : vk::ImageLayout::UNDEFINED,
/// }
/// ```
///
/// See [VkAttachmentDescription](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkAttachmentDescription.html) for more detail.
///
#[derive(Debug, Clone)]
pub struct AttachmentDescCI {
    inner: vk::AttachmentDescription,
}

impl VulkanCI<vk::AttachmentDescription> for AttachmentDescCI {

    fn default_ci() -> vk::AttachmentDescription {

        vk::AttachmentDescription {
            flags : vk::AttachmentDescriptionFlags::empty(),
            format: vk::Format::UNDEFINED,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op : vk::AttachmentLoadOp::DONT_CARE,
            store_op: vk::AttachmentStoreOp::DONT_CARE,
            stencil_load_op : vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout  : vk::ImageLayout::UNDEFINED,
        }
    }
}

impl AsRef<vk::AttachmentDescription> for AttachmentDescCI {

    fn as_ref(&self) -> &vk::AttachmentDescription {
        &self.inner
    }
}

impl AttachmentDescCI {

    /// Initialize `vk::AttachmentDescription` with default value.
    ///
    /// `format` is the format of image view used by this attachment.
    pub fn new(format: vk::Format) -> AttachmentDescCI {

        AttachmentDescCI {
            inner: vk::AttachmentDescription {
                format,
                ..AttachmentDescCI::default_ci()
            }
        }
    }

    /// Set the `samples` member of `vk::AttachmentDescription`.
    ///
    /// `count` specifies the number of samples for the image.
    #[inline(always)]
    pub fn sample_count(mut self, count: vk::SampleCountFlags) -> AttachmentDescCI {
        self.inner.samples = count; self
    }

    /// Set the `load_op` and `store_op` members of `vk::AttachmentDescription`.
    ///
    /// `load` specifies how to treat the color or depth attachment at the beginning of the subpass.
    ///
    /// `store` specifies how to treat the color or depth attachment at the end of subpass.
    #[inline(always)]
    pub fn op(mut self, load: vk::AttachmentLoadOp, store: vk::AttachmentStoreOp) -> AttachmentDescCI {
        self.inner.load_op  = load;
        self.inner.store_op = store; self
    }

    /// Set the `stencil_load_op` and `stencil_store_op` members of `vk::AttachmentDescription`.
    ///
    /// `load` specifies how to treat the stencil attachment at the beginning of the subpass.
    ///
    /// `store` specifies how to treat the stencil attachment at the end of subpass.
    #[inline(always)]
    pub fn stencil_op(mut self, load: vk::AttachmentLoadOp, store: vk::AttachmentStoreOp) -> AttachmentDescCI {
        self.inner.stencil_load_op  = load;
        self.inner.stencil_store_op = store; self
    }

    /// Set the `initial_layout` and `final_layout` members of `vk::AttachmentDescription`.
    ///
    /// `initial` is the layout of the attachment image before the render pass begins.
    ///
    /// `final` is the layout of the attachment image that will be transitioned to after the render pass.
    #[inline(always)]
    pub fn layout(mut self, initial: vk::ImageLayout, final_: vk::ImageLayout) -> AttachmentDescCI {
        self.inner.initial_layout = initial;
        self.inner.final_layout   = final_; self
    }

    /// Set the `flags` member for `vk::AttachmentDescription`.
    ///
    /// `flags` specifies additional properties of the attachment.
    #[inline(always)]
    pub fn flags(mut self, flags: vk::AttachmentDescriptionFlags) -> AttachmentDescCI {
        self.inner.flags = flags; self
    }
}

impl From<AttachmentDescCI> for vk::AttachmentDescription {

    fn from(v: AttachmentDescCI) -> vk::AttachmentDescription {
        v.inner
    }
}

impl From<vk::AttachmentDescription> for AttachmentDescCI {

    fn from(v: vk::AttachmentDescription) -> AttachmentDescCI {
        AttachmentDescCI { inner: v }
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
/// Wrapper class for `vk::SubpassDescription`.
///
/// The default values are defined as follows:
/// ``` ignore
/// vk::SubpassDescription {
///     flags: vk::SubpassDescriptionFlags::empty(),
///     pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
///     input_attachment_count: 0,
///     p_input_attachments   : ptr::null(),
///     color_attachment_count: 0,
///     p_color_attachments   : ptr::null(),
///     p_resolve_attachments : ptr::null(),
///     preserve_attachment_count: 0,
///     p_preserve_attachments   : ptr::null(),
///     p_depth_stencil_attachment: ptr::null(),
/// }
/// ```
///
/// See [VkSubpassDescription](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkSubpassDescription.html) for more detail.
///
#[derive(Debug)]
pub struct SubpassDescCI {
    inner: vk::SubpassDescription,

    inputs   : Option<Vec<vk::AttachmentReference>>,
    colors   : Option<Vec<vk::AttachmentReference>>,
    resolves : Option<Vec<vk::AttachmentReference>>,
    preserves: Option<Vec<vkuint>>,
    depth_stencil: Option<Vec<vk::AttachmentReference>>,
}

impl VulkanCI<vk::SubpassDescription> for SubpassDescCI {

    fn default_ci() -> vk::SubpassDescription {

        vk::SubpassDescription {
            flags: vk::SubpassDescriptionFlags::empty(),
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            input_attachment_count: 0,
            p_input_attachments   : ptr::null(),
            color_attachment_count: 0,
            p_color_attachments   : ptr::null(),
            p_resolve_attachments : ptr::null(),
            preserve_attachment_count: 0,
            p_preserve_attachments   : ptr::null(),
            p_depth_stencil_attachment: ptr::null(),
        }
    }
}

impl AsRef<vk::SubpassDescription> for SubpassDescCI {

    fn as_ref(&self) -> &vk::SubpassDescription {
        &self.inner
    }
}

impl SubpassDescCI {

    /// Initialize `vk::SubpassDescription` with default value.
    ///
    /// `bind_point` specifies the pipeline type of this subpass.
    pub fn new(bind_point: vk::PipelineBindPoint) -> SubpassDescCI {

        SubpassDescCI {
            inner: vk::SubpassDescription {
                pipeline_bind_point: bind_point,
                ..SubpassDescCI::default_ci()
            },
            inputs   : None,
            colors   : None,
            resolves : None,
            preserves: None,
            depth_stencil: None,
        }
    }

    /// Add input attachment to this subpass.
    ///
    /// `attachment_index` is the corresponding index of attachment defined in `vk::RenderPassCreateInfo`.
    ///
    /// `image_layout` specifies the layout of attachment image of this subpass.
    #[inline]
    pub fn add_input_attachment(mut self, attachment_index: vkuint, image_layout: vk::ImageLayout) -> SubpassDescCI {

        let inputs = self.inputs.get_or_insert(Vec::new());
        inputs.push(vk::AttachmentReference {
            attachment: attachment_index,
            layout: image_layout
        });

        self.inner.input_attachment_count = inputs.len() as _;
        self.inner.p_input_attachments    = inputs.as_ptr(); self
    }

    /// Add color attachment to this subpass.
    ///
    /// `attachment_index` is the corresponding index of attachment defined in `vk::RenderPassCreateInfo`.
    ///
    /// `image_layout` specifies the layout of attachment image of this subpass.
    #[inline]
    pub fn add_color_attachment(mut self, attachment_index: vkuint, image_layout: vk::ImageLayout) -> SubpassDescCI {

        let colors = self.colors.get_or_insert(Vec::new());
        colors.push(vk::AttachmentReference {
            attachment: attachment_index,
            layout: image_layout,
        });

        self.inner.color_attachment_count = colors.len() as _;
        self.inner.p_color_attachments    = colors.as_ptr(); self
    }

    /// Add resolve attachment to this subpass.
    ///
    /// `attachment_index` is the corresponding index of attachment defined in `vk::RenderPassCreateInfo`.
    ///
    /// `image_layout` specifies the layout of attachment image of this subpass.
    #[inline]
    pub fn add_resolve_attachment(mut self, attachment_index: vkuint, image_layout: vk::ImageLayout) -> SubpassDescCI {

        let resolves = self.resolves.get_or_insert(Vec::new());
        resolves.push(vk::AttachmentReference {
            attachment: attachment_index,
            layout: image_layout,
        });

        self.inner.preserve_attachment_count = resolves.len() as _;
        self.inner.p_resolve_attachments     = resolves.as_ptr(); self
    }

    /// Add preserve attachment to this subpass.
    ///
    /// `attachment_index` is the corresponding index of attachment defined in `vk::RenderPassCreateInfo`.
    #[inline(always)]
    pub fn add_preserve_attachment(mut self, attachment_index: vkuint) -> SubpassDescCI {

        let preserves = self.preserves.get_or_insert(Vec::new());
        preserves.push(attachment_index);

        self.inner.p_preserve_attachments = preserves.as_ptr(); self
    }

    /// Set depth stencil attachment of this subpass.
    ///
    /// `attachment_index` is the corresponding index of attachment defined in `vk::RenderPassCreateInfo`.
    ///
    /// `image_layout` specifies the layout of attachment image of this subpass.
    pub fn set_depth_stencil_attachment(mut self, attachment_index: vkuint, image_layout: vk::ImageLayout) -> SubpassDescCI {

        let depth_stencil_ref = self.depth_stencil.get_or_insert(vec![vk::AttachmentReference::default()]);
        depth_stencil_ref[0] = vk::AttachmentReference {
            attachment: attachment_index,
            layout: image_layout,
        };

        self.inner.p_depth_stencil_attachment = depth_stencil_ref.as_ptr(); self
    }

    /// Set the `flags` member for `vk::SubpassDescription`.
    ///
    /// It specifies usage of the subpass.
    #[inline(always)]
    pub fn flags(mut self, flags: vk::SubpassDescriptionFlags) -> SubpassDescCI {
        self.inner.flags = flags; self
    }
}
// ----------------------------------------------------------------------------------------------


// ----------------------------------------------------------------------------------------------
/// Wrapper class for `vk::SubpassDependency`.
///
/// The default values are defined as follows:
/// ``` ignore
/// vk::SubpassDependency {
///     src_subpass: vk::SUBPASS_EXTERNAL,
///     dst_subpass: vk::SUBPASS_EXTERNAL,
///     src_stage_mask   : vk::PipelineStageFlags::empty(),
///     dst_stage_mask   : vk::PipelineStageFlags::empty(),
///     src_access_mask  : vk::AccessFlags::empty(),
///     dst_access_mask  : vk::AccessFlags::empty(),
///     dependency_flags : vk::DependencyFlags::empty(),
/// }
/// ```
///
/// See [VkSubpassDependency](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkSubpassDependency.html) for more detail.
///
#[derive(Debug, Clone)]
pub struct SubpassDependencyCI {
    inner: vk::SubpassDependency,
}

impl VulkanCI<vk::SubpassDependency> for SubpassDependencyCI {

    fn default_ci() -> vk::SubpassDependency {

        vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: vk::SUBPASS_EXTERNAL,
            src_stage_mask   : vk::PipelineStageFlags::empty(),
            dst_stage_mask   : vk::PipelineStageFlags::empty(),
            src_access_mask  : vk::AccessFlags::empty(),
            dst_access_mask  : vk::AccessFlags::empty(),
            dependency_flags : vk::DependencyFlags::empty(),
        }
    }
}

impl AsRef<vk::SubpassDependency> for SubpassDependencyCI {

    fn as_ref(&self) -> &vk::SubpassDependency {
        &self.inner
    }
}

impl SubpassDependencyCI {

    /// Initialize `vk::SubpassDependency` with default value.
    ///
    /// `src` specifies the subpass index of the first subpass, or set to `vk::SUBPASS_EXTERNAL`.
    ///
    /// `dst` specifies the subpass index of the second subpass, or set to `vk::SUBPASS_EXTERNAL`.
    pub fn new(src: vkuint, dst: vkuint) -> SubpassDependencyCI {

        SubpassDependencyCI {
            inner: vk::SubpassDependency {
                src_subpass: src,
                dst_subpass: dst,
                ..SubpassDependencyCI::default_ci()
            },
        }
    }

    /// Set the `src_stage_mask` and `dst_stage_mask` members for `vk::SubpassDependency`.
    ///
    /// `src` specifies the source stage mask.
    ///
    /// `dst` specifies the destination stage mask.
    #[inline(always)]
    pub fn stage_mask(mut self, src: vk::PipelineStageFlags, dst: vk::PipelineStageFlags) -> SubpassDependencyCI {
        self.inner.src_stage_mask = src;
        self.inner.dst_stage_mask = dst; self
    }

    /// Set the `src_access_mask` and `dst_access_mask` members for `vk::SubpassDependency`.
    ///
    /// `src` specifies the source access mask.
    ///
    /// `dst` specifies the destination access mask.
    #[inline(always)]
    pub fn access_mask(mut self, src: vk::AccessFlags, dst: vk::AccessFlags) -> SubpassDependencyCI {
        self.inner.src_access_mask = src;
        self.inner.dst_access_mask = dst; self
    }

    /// Set the `flags` member for `vk::SubpassDependency`.
    ///
    /// It specifies some additional parameters of the subpass dependency.
    #[inline(always)]
    pub fn flags(mut self, flags: vk::DependencyFlags) -> SubpassDependencyCI {
        self.inner.dependency_flags = flags; self
    }
}

impl From<SubpassDependencyCI> for vk::SubpassDependency {

    fn from(v: SubpassDependencyCI) -> vk::SubpassDependency {
        v.inner
    }
}

impl From<vk::SubpassDependency> for SubpassDependencyCI {

    fn from(v: vk::SubpassDependency) -> SubpassDependencyCI {
        SubpassDependencyCI { inner: v }
    }
}
// ----------------------------------------------------------------------------------------------

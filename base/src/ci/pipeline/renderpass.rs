
use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::VkDevice;
use crate::context::VkObjectDiscardable;

use crate::ci::{VulkanCI, VkObjectBuildableCI};

use crate::error::{VkResult, VkError};
use crate::vkuint;

use std::ptr;
use std::ops::Deref;

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::RenderPassBeginInfo.
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

impl Deref for RenderPassBI {
    type Target = vk::RenderPassBeginInfo;

    fn deref(&self) -> &vk::RenderPassBeginInfo {
        &self.inner
    }
}

impl RenderPassBI {

    pub fn new(render_pass: vk::RenderPass, framebuffer: vk::Framebuffer) -> RenderPassBI {

        RenderPassBI {
            inner: vk::RenderPassBeginInfo {
                render_pass, framebuffer,
                ..RenderPassBI::default_ci()
            },
            clears: None,
        }
    }

    #[inline(always)]
    pub fn render_extent(mut self, area: vk::Extent2D) -> RenderPassBI {
        self.inner.render_area.extent = area; self
    }

    #[inline(always)]
    pub fn render_area_offset(mut self, offset: vk::Offset2D) -> RenderPassBI {
        self.inner.render_area.offset = offset; self
    }

    #[inline]
    pub fn add_clear_value(mut self, value: vk::ClearValue) -> RenderPassBI {

        let clear_values = self.clears.get_or_insert(Vec::new());
        clear_values.push(value);

        self.inner.clear_value_count = clear_values.len() as _;
        self.inner.p_clear_values    = clear_values.as_ptr(); self
    }

    #[inline]
    pub fn set_clear_values(mut self, values: Vec<vk::ClearValue>) -> RenderPassBI {

        self.inner.clear_value_count = values.len() as _;
        self.inner.p_clear_values    = values.as_ptr();

        self.clears.replace(values); self
    }
}
// ----------------------------------------------------------------------------------------------


// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::RenderPassCreateInfo.
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

impl Deref for RenderPassCI {
    type Target = vk::RenderPassCreateInfo;

    fn deref(&self) -> &vk::RenderPassCreateInfo {
        &self.inner
    }
}

impl VkObjectBuildableCI for RenderPassCI {
    type ObjectType = vk::RenderPass;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let render_pass = unsafe {
            device.logic.handle.create_render_pass(self, None)
                .map_err(|_| VkError::create("Render Pass"))?
        };
        Ok(render_pass)
    }
}

impl RenderPassCI {

    pub fn new() -> RenderPassCI {

        RenderPassCI {
            inner: RenderPassCI::default_ci(),
            attachments : Vec::new(),
            subpasses   : Vec::new(),
            dependencies: None,
            subpass_cis : Vec::new(),
        }
    }

    #[inline]
    pub fn add_attachment(mut self, attachment: AttachmentDescCI) -> RenderPassCI {

        self.attachments.push(attachment.into());
        self.inner.attachment_count = self.attachments.len() as _;
        self.inner.p_attachments    = self.attachments.as_ptr(); self
    }

    #[inline]
    pub fn add_subpass(mut self, subpass: SubpassDescCI) -> RenderPassCI {

        if let Some(ref resolves) = subpass.resolves {
            assert_eq!(resolves.len() as vkuint, subpass.color_attachment_count);
        }

        self.subpasses.push(subpass.inner.clone());
        self.subpass_cis.push(subpass);

        self.inner.subpass_count = self.subpasses.len() as _;
        self.inner.p_subpasses   = self.subpasses.as_ptr(); self
    }

    #[inline]
    pub fn add_dependency(mut self, dependency: SubpassDependencyCI) -> RenderPassCI {

        let dependencies = self.dependencies.get_or_insert(Vec::new());
        dependencies.push(dependency.into());

        self.inner.dependency_count = dependencies.len() as _;
        self.inner.p_dependencies   = dependencies.as_ptr(); self
    }

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
/// Wrapper class for vk::AttachmentDescription.
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

impl Deref for AttachmentDescCI {
    type Target = vk::AttachmentDescription;

    fn deref(&self) -> &vk::AttachmentDescription {
        &self.inner
    }
}

impl AttachmentDescCI {

    pub fn new(format: vk::Format) -> AttachmentDescCI {

        AttachmentDescCI {
            inner: vk::AttachmentDescription {
                format,
                ..AttachmentDescCI::default_ci()
            }
        }
    }

    #[inline(always)]
    pub fn sample_count(mut self, count: vk::SampleCountFlags) -> AttachmentDescCI {
        self.inner.samples = count; self
    }

    #[inline(always)]
    pub fn op(mut self, load: vk::AttachmentLoadOp, store: vk::AttachmentStoreOp) -> AttachmentDescCI {
        self.inner.load_op  = load;
        self.inner.store_op = store; self
    }

    #[inline(always)]
    pub fn stencil_op(mut self, load: vk::AttachmentLoadOp, store: vk::AttachmentStoreOp) -> AttachmentDescCI {
        self.inner.stencil_load_op  = load;
        self.inner.stencil_store_op = store; self
    }

    #[inline(always)]
    pub fn layout(mut self, initial: vk::ImageLayout, r#final: vk::ImageLayout) -> AttachmentDescCI {
        self.inner.initial_layout = initial;
        self.inner.final_layout   = r#final; self
    }

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
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::SubpassDescription.
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

impl Deref for SubpassDescCI {
    type Target = vk::SubpassDescription;

    fn deref(&self) -> &vk::SubpassDescription {
        &self.inner
    }
}

impl SubpassDescCI {

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

    #[inline(always)]
    pub fn add_preserve_attachment(mut self, attachment_index: vkuint) -> SubpassDescCI {

        let preserves = self.preserves.get_or_insert(Vec::new());
        preserves.push(attachment_index);

        self.inner.p_preserve_attachments = preserves.as_ptr(); self
    }

    pub fn set_depth_stencil_attachment(mut self, attachment_index: vkuint, image_layout: vk::ImageLayout) -> SubpassDescCI {

        let depth_stencil_ref = self.depth_stencil.get_or_insert(vec![vk::AttachmentReference::default()]);
        depth_stencil_ref[0] = vk::AttachmentReference {
            attachment: attachment_index,
            layout: image_layout,
        };

        self.inner.p_depth_stencil_attachment = depth_stencil_ref.as_ptr(); self
    }

    #[inline(always)]
    pub fn flags(mut self, flags: vk::SubpassDescriptionFlags) -> SubpassDescCI {
        self.inner.flags = flags; self
    }
}
// ----------------------------------------------------------------------------------------------


// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::SubpassDependency.
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

impl Deref for SubpassDependencyCI {
    type Target = vk::SubpassDependency;

    fn deref(&self) -> &vk::SubpassDependency {
        &self.inner
    }
}

impl SubpassDependencyCI {

    pub fn new(src: vkuint, dst: vkuint) -> SubpassDependencyCI {

        SubpassDependencyCI {
            inner: vk::SubpassDependency {
                src_subpass: src,
                dst_subpass: dst,
                ..SubpassDependencyCI::default_ci()
            },
        }
    }

    #[inline(always)]
    pub fn stage_mask(mut self, src: vk::PipelineStageFlags, dst: vk::PipelineStageFlags) -> SubpassDependencyCI {
        self.inner.src_stage_mask = src;
        self.inner.dst_stage_mask = dst; self
    }

    #[inline(always)]
    pub fn access_mask(mut self, src: vk::AccessFlags, dst: vk::AccessFlags) -> SubpassDependencyCI {
        self.inner.src_access_mask = src;
        self.inner.dst_access_mask = dst; self
    }

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
// ----------------------------------------------------------------------------------------------

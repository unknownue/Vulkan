
use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::VkDevice;
use crate::context::VkObjectCreatable;
use crate::ci::{VulkanCI, VkObjectBuildableCI};
use crate::error::{VkResult, VkError};
use crate::vkuint;

use std::ptr;

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::RenderPassBeginInfo.
#[derive(Clone)]
pub struct RenderPassBI {

    bi: vk::RenderPassBeginInfo,
    clears: Vec<vk::ClearValue>,
}

impl VulkanCI for RenderPassBI {
    type CIType = vk::RenderPassBeginInfo;

    fn default_ci() -> Self::CIType {

        vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            p_next: ptr::null(),
            render_area: vk::Rect2D {
                extent: vk::Extent2D {
                    width : 0,
                    height: 0,
                },
                offset: vk::Offset2D { x: 0, y: 0 },
            },
            clear_value_count: 0,
            p_clear_values   : ptr::null(),
            render_pass: vk::RenderPass::null(),
            framebuffer: vk::Framebuffer::null(),
        }
    }
}

impl RenderPassBI {

    pub fn new(render_pass: vk::RenderPass, framebuffer: vk::Framebuffer) -> RenderPassBI {

        RenderPassBI {
            bi: vk::RenderPassBeginInfo {
                render_pass, framebuffer,
                ..RenderPassBI::default_ci()
            },
            clears: Vec::new(),
        }
    }

    pub fn render_extent(mut self, area: vk::Extent2D) -> RenderPassBI {
        self.bi.render_area.extent = area; self
    }

    pub fn render_area_offset(mut self, offset: vk::Offset2D) -> RenderPassBI {
        self.bi.render_area.offset = offset; self
    }

    pub fn clear_values(mut self, values: &[vk::ClearValue]) -> RenderPassBI {
        self.clears.extend_from_slice(values); self
    }

    pub(crate) fn value(&self) -> vk::RenderPassBeginInfo {

        vk::RenderPassBeginInfo {
            clear_value_count: self.clears.len() as _,
            p_clear_values   : self.clears.as_ptr(),
            ..self.bi
        }
    }
}
// ----------------------------------------------------------------------------------------------


// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::RenderPassCreateInfo.
#[derive(Debug, Clone)]
pub struct RenderPassCI {

    ci: vk::RenderPassCreateInfo,
    attachments : Vec<vk::AttachmentDescription>,
    subpasses   : Vec<vk::SubpassDescription>,
    dependencies: Vec<vk::SubpassDependency>,
}

impl VulkanCI for RenderPassCI {
    type CIType = vk::RenderPassCreateInfo;

    fn default_ci() -> Self::CIType {

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

impl VkObjectBuildableCI for RenderPassCI {
    type ObjectType = vk::RenderPass;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let renderpass_ci = vk::RenderPassCreateInfo {
            attachment_count: self.attachments.len() as _,
            p_attachments   : self.attachments.as_ptr(),
            subpass_count   : self.subpasses.len() as _,
            p_subpasses     : self.subpasses.as_ptr(),
            dependency_count: self.dependencies.len() as _,
            p_dependencies  : self.dependencies.as_ptr(),
            ..self.ci
        };

        let render_pass = unsafe {
            device.logic.handle.create_render_pass(&renderpass_ci, None)
                .map_err(|_| VkError::create("Render Pass"))?
        };
        Ok(render_pass)
    }
}

impl RenderPassCI {

    pub fn new() -> RenderPassCI {

        RenderPassCI {
            ci: RenderPassCI::default_ci(),
            attachments : Vec::new(),
            subpasses   : Vec::new(),
            dependencies: Vec::new(),
        }
    }

    pub fn add_attachment(mut self, attachment: vk::AttachmentDescription) -> RenderPassCI {
        self.attachments.push(attachment); self
    }

    pub fn add_subpass(mut self, subpass: vk::SubpassDescription) -> RenderPassCI {
        self.subpasses.push(subpass); self
    }

    pub fn add_dependency(mut self, dependency: vk::SubpassDependency) -> RenderPassCI {
        self.dependencies.push(dependency); self
    }

    pub fn flags(mut self, flags: vk::RenderPassCreateFlags) -> RenderPassCI {
        self.ci.flags = flags; self
    }
}

impl VkObjectCreatable for vk::RenderPass {

    fn discard(self, device: &VkDevice) {
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
    ci: vk::AttachmentDescription,
}

impl VulkanCI for AttachmentDescCI {
    type CIType = vk::AttachmentDescription;

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

impl AttachmentDescCI {

    pub fn new(format: vk::Format) -> AttachmentDescCI {

        AttachmentDescCI {
            ci: vk::AttachmentDescription {
                format,
                ..AttachmentDescCI::default_ci()
            }
        }
    }

    pub fn value(&self) -> vk::AttachmentDescription {
        self.ci.clone()
    }

    pub fn sample_count(mut self, count: vk::SampleCountFlags) -> AttachmentDescCI {
        self.ci.samples = count; self
    }

    pub fn op(mut self, load: vk::AttachmentLoadOp, store: vk::AttachmentStoreOp) -> AttachmentDescCI {
        self.ci.load_op  = load;
        self.ci.store_op = store; self
    }

    pub fn stencil_op(mut self, load: vk::AttachmentLoadOp, store: vk::AttachmentStoreOp) -> AttachmentDescCI {
        self.ci.stencil_load_op  = load;
        self.ci.stencil_store_op = store; self
    }

    pub fn layout(mut self, initial: vk::ImageLayout, r#final: vk::ImageLayout) -> AttachmentDescCI {
        self.ci.initial_layout = initial;
        self.ci.final_layout   = r#final; self
    }

    pub fn flags(mut self, flags: vk::AttachmentDescriptionFlags) -> AttachmentDescCI {
        self.ci.flags = flags; self
    }
}

impl From<AttachmentDescCI> for vk::AttachmentDescription {

    fn from(value: AttachmentDescCI) -> vk::AttachmentDescription {
        value.ci
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::SubpassDescription.
#[derive(Debug)]
pub struct SubpassDescCI {
    ci: vk::SubpassDescription,

    inputs   : Vec<vk::AttachmentReference>,
    colors   : Vec<vk::AttachmentReference>,
    resolves : Vec<vk::AttachmentReference>,
    preserves: Vec<vkuint>,
    depth_stencil: Vec<vk::AttachmentReference>,
}

impl VulkanCI for SubpassDescCI {
    type CIType = vk::SubpassDescription;

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

impl SubpassDescCI {

    pub fn new(bind_point: vk::PipelineBindPoint) -> SubpassDescCI {

        SubpassDescCI {
            ci: vk::SubpassDescription {
                pipeline_bind_point: bind_point,
                ..SubpassDescCI::default_ci()
            },
            inputs   : Vec::new(),
            colors   : Vec::new(),
            resolves : Vec::new(),
            preserves: Vec::new(),
            depth_stencil: Vec::new(),
        }
    }

    pub fn value(&self) -> vk::SubpassDescription {

        vk::SubpassDescription {
            input_attachment_count: self.inputs.len() as _,
            p_input_attachments   : self.inputs.as_ptr(),
            color_attachment_count: self.colors.len() as _,
            p_color_attachments   : self.colors.as_ptr(),
            p_resolve_attachments : if self.resolves.is_empty() { ptr::null() } else { self.resolves.as_ptr() },
            preserve_attachment_count: self.preserves.len() as _,
            p_preserve_attachments   : self.preserves.as_ptr(),
            p_depth_stencil_attachment: if self.depth_stencil.is_empty() { ptr::null() } else { self.depth_stencil.as_ptr() },
            ..self.ci
        }
    }

    pub fn add_input_attachment(mut self, attachment_index: vkuint, image_layout: vk::ImageLayout) -> SubpassDescCI {
        self.inputs.push(vk::AttachmentReference {
            attachment: attachment_index,
            layout: image_layout
        }); self
    }

    pub fn add_color_attachment(mut self, attachment_index: vkuint, image_layout: vk::ImageLayout) -> SubpassDescCI {
        self.colors.push(vk::AttachmentReference {
            attachment: attachment_index,
            layout: image_layout,
        }); self
    }

    pub fn add_resolve_attachment(mut self, attachment_index: vkuint, image_layout: vk::ImageLayout) -> SubpassDescCI {
        self.resolves.push(vk::AttachmentReference {
            attachment: attachment_index,
            layout: image_layout,
        }); self
    }

    pub fn add_preserve_attachment(mut self, attachment_index: vkuint) -> SubpassDescCI {
        self.preserves.push(attachment_index); self
    }

    pub fn set_depth_stencil_attachment(mut self, attachment_index: vkuint, image_layout: vk::ImageLayout) -> SubpassDescCI {

        let depth_stencil_ref = vk::AttachmentReference {
            attachment: attachment_index,
            layout: image_layout,
        };
        if self.depth_stencil.is_empty() {
            self.depth_stencil.push(depth_stencil_ref);
        } else {
            self.depth_stencil[0] = depth_stencil_ref;
        }
        self
    }

    pub fn flags(mut self, flags: vk::SubpassDescriptionFlags) -> SubpassDescCI {
        self.ci.flags = flags; self
    }
}
// ----------------------------------------------------------------------------------------------


// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::SubpassDependency.
#[derive(Debug, Clone)]
pub struct SubpassDependencyCI {
    ci: vk::SubpassDependency,
}

impl VulkanCI for SubpassDependencyCI {
    type CIType = vk::SubpassDependency;

    fn default_ci() -> Self::CIType {

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

impl SubpassDependencyCI {

    pub fn new(src: vkuint, dst: vkuint) -> SubpassDependencyCI {

        SubpassDependencyCI {
            ci: vk::SubpassDependency {
                src_subpass: src,
                dst_subpass: dst,
                ..SubpassDependencyCI::default_ci()
            },
        }
    }

    pub fn value(&self) -> vk::SubpassDependency {
        self.ci.clone()
    }

    pub fn stage_mask(mut self, src: vk::PipelineStageFlags, dst: vk::PipelineStageFlags) -> SubpassDependencyCI {
        self.ci.src_stage_mask = src;
        self.ci.dst_stage_mask = dst; self
    }

    pub fn access_mask(mut self, src: vk::AccessFlags, dst: vk::AccessFlags) -> SubpassDependencyCI {
        self.ci.src_access_mask = src;
        self.ci.dst_access_mask = dst; self
    }

    pub fn flags(mut self, flags: vk::DependencyFlags) -> SubpassDependencyCI {
        self.ci.dependency_flags = flags; self
    }
}

impl From<SubpassDependencyCI> for vk::SubpassDependency {

    fn from(value: SubpassDependencyCI) -> vk::SubpassDependency {
        value.ci
    }
}
// ----------------------------------------------------------------------------------------------

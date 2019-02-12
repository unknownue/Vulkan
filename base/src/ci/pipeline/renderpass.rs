
use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::VkDevice;
use crate::context::VkObjectCreatable;
use crate::ci::{VulkanCI, VkObjectBuildableCI};
use crate::error::{VkResult, VkError};

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

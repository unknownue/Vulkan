
pub use self::renderpass::{RenderPassCI, RenderPassBI};

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
use crate::context::VkObjectCreatable;
use crate::ci::{VulkanCI, VkObjectBuildableCI};
use crate::error::{VkResult, VkError};

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

impl VkObjectCreatable for vk::PipelineLayout {

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

impl VkObjectCreatable for vk::Framebuffer {

    fn discard(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_framebuffer(self, None);
        }
    }
}

impl VkObjectCreatable for &Vec<vk::Framebuffer> {

    fn discard(self, device: &VkDevice) {

        for framebuffer in self {
            device.discard(*framebuffer);
        }
    }
}
// ---------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
// Wrapper class for vk::FramebufferCreateInfo.


use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::VkDevice;
use crate::context::VkObjectCreatable;
use crate::ci::VulkanCI;
use crate::error::{VkResult, VkError};

use std::ptr;

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::RenderPassBeginInfo.
#[derive(Clone)]
pub struct RenderPassBI {

    bi: vk::RenderPassBeginInfo,
    clears: Vec<vk::ClearValue>,
}

impl VulkanCI<vk::RenderPassBeginInfo> for RenderPassBI {

    fn default_ci() -> vk::RenderPassBeginInfo {

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

    pub(crate) fn build(&self) -> vk::RenderPassBeginInfo {

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

impl RenderPassCI {

    pub fn new() -> RenderPassCI {

        RenderPassCI {
            ci: RenderPassCI::default_ci(),
            attachments : Vec::new(),
            subpasses   : Vec::new(),
            dependencies: Vec::new(),
        }
    }

    pub fn build(mut self, device: &VkDevice) -> VkResult<vk::RenderPass> {

        self.ci.attachment_count = self.attachments.len() as _;
        self.ci.p_attachments    = self.attachments.as_ptr();

        self.ci.subpass_count = self.subpasses.len() as _;
        self.ci.p_subpasses   = self.subpasses.as_ptr();

        self.ci.dependency_count = self.dependencies.len() as _;
        self.ci.p_dependencies   = self.dependencies.as_ptr();

        let render_pass = unsafe {
            device.logic.handle.create_render_pass(&self.ci, None)
                .map_err(|_| VkError::create("Render Pass"))?
        };
        Ok(render_pass)
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
/// Wrapper class for vk::PipelineLayoutCreateInfo.
#[derive(Debug, Clone)]
pub struct PipelineLayoutCI {

    ci: vk::PipelineLayoutCreateInfo,
    set_layouts   : Vec<vk::DescriptorSetLayout>,
    push_constants: Vec<vk::PushConstantRange>,
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

impl PipelineLayoutCI {

    pub fn new() -> PipelineLayoutCI {

        PipelineLayoutCI {
            ci: PipelineLayoutCI::default_ci(),
            set_layouts    : Vec::new(),
            push_constants : Vec::new(),
        }
    }

    pub fn build(mut self, device: &VkDevice) -> VkResult<vk::PipelineLayout> {

        self.ci.set_layout_count = self.set_layouts.len() as _;
        self.ci.p_set_layouts    = self.set_layouts.as_ptr();

        self.ci.push_constant_range_count = self.push_constants.len() as _;
        self.ci.p_push_constant_ranges    = self.push_constants.as_ptr();

        let pipeline_layout = unsafe {
            device.logic.handle.create_pipeline_layout(&self.ci, None)
                .map_err(|_| VkError::create("Pipeline Layout"))?
        };
        Ok(pipeline_layout)
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

    pub fn build(mut self, device: &VkDevice) -> VkResult<vk::Framebuffer> {

        self.ci.attachment_count = self.attachments.len() as _;
        self.ci.p_attachments    = self.attachments.as_ptr();

        let framebuffer = unsafe {
            device.logic.handle.create_framebuffer(&self.ci, None)
                .map_err(|_| VkError::create("Framebuffer"))?
        };
        Ok(framebuffer)
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

impl<T> VkObjectCreatable for &T where T: AsRef<[vk::Framebuffer]> {

    fn discard(self, device: &VkDevice) {

        for framebuffer in self.as_ref() {
            device.discard(*framebuffer);
        }
    }
}
// ---------------------------------------------------------------------------------------------

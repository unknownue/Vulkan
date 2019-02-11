
use ash::vk;

use crate::ci::VulkanCI;

use std::ptr;

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::RenderPassBeginInfo.
#[derive(Clone)]
pub struct RenderPassBI {

    bi: vk::RenderPassBeginInfo,
    clears: Vec<vk::ClearValue>,
}

impl VulkanCI<vk::RenderPassBeginInfo> for RenderPassBI {

    fn inner_default() -> RenderPassBI {

        RenderPassBI {
            bi: vk::RenderPassBeginInfo {
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
            },
            clears: Vec::new(),
        }
    }
}

impl RenderPassBI {

    pub fn new(render_pass: vk::RenderPass, framebuffer: vk::Framebuffer) -> RenderPassBI {

        RenderPassBI {
            bi: vk::RenderPassBeginInfo {
                render_pass, framebuffer,
                ..RenderPassBI::inner_default().bi
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

impl From<RenderPassBI> for vk::RenderPassBeginInfo {

    fn from(value: RenderPassBI) -> vk::RenderPassBeginInfo {
        value.bi
    }
}
// ----------------------------------------------------------------------------------------------


use ash::vk;

use std::ptr;

/// Wrapper class for vk::RenderPassBeginInfo.
pub struct RenderPassBI {

    bi: vk::RenderPassBeginInfo,
    clears: Vec<vk::ClearValue>,
}

impl RenderPassBI {

    pub fn new(render_pass: vk::RenderPass, framebuffer: vk::Framebuffer) -> RenderPassBI {

        let begin_bi = vk::RenderPassBeginInfo {
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
            render_pass, framebuffer,
        };

        RenderPassBI {
            bi: begin_bi,
            clears: Vec::new(),
        }
    }

    pub fn render_extent(mut self, area: vk::Extent2D) -> RenderPassBI {
        self.bi.render_area.extent = area;
        self
    }

    pub fn render_area_offset(mut self, offset: vk::Offset2D) -> RenderPassBI {
        self.bi.render_area.offset = offset;
        self
    }

    pub fn clear_values(mut self, values: &[vk::ClearValue]) -> RenderPassBI {
        self.clears.extend_from_slice(values);
        self
    }

    pub(crate) fn build(&self) -> vk::RenderPassBeginInfo {

        vk::RenderPassBeginInfo {
            clear_value_count: self.clears.len() as _,
            p_clear_values   : self.clears.as_ptr(),
            ..self.bi
        }
    }
}

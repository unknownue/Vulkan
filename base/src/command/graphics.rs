
use ash::vk;
use ash::version::DeviceV1_0;

use crate::command::VkCommandType;
use crate::command::recorder::VkCmdRecorder;
use crate::{vkuint, vkfloat, vksint, vkbytes};

use crate::ci::pipeline::RenderPassBI;


pub struct IGraphics;

impl VkCommandType for IGraphics {
    const BIND_POINT: vk::PipelineBindPoint = vk::PipelineBindPoint::GRAPHICS;
}

impl<'a> CmdGraphicsApi for VkCmdRecorder<'a, IGraphics> {

    fn begin_render_pass(&self, bi: RenderPassBI) -> &VkCmdRecorder<'a, IGraphics> {

        // Currently only use primary command buffer, so always set vk::SubpassContents::INLINE here.
        unsafe {
            self.device.handle.cmd_begin_render_pass(self.command, &(*bi), vk::SubpassContents::INLINE);
        } self
    }

    /// Set the viewport dynamically.
    fn set_viewport(&self, first_viewport: vkuint, viewports: &[vk::Viewport]) -> &VkCmdRecorder<'a, IGraphics> {
        unsafe {
            self.device.handle.cmd_set_viewport(self.command, first_viewport, viewports);
        } self
    }

    /// Set the scissor rectangles dynamically.
    fn set_scissor(&self, first_scissor: vkuint, scissors: &[vk::Rect2D]) -> &VkCmdRecorder<'a, IGraphics> {
        unsafe {
            self.device.handle.cmd_set_scissor(self.command, first_scissor, &scissors);
        } self
    }

    /// Set the line width dynamically.
    fn set_line_width(&self, width: vkfloat) -> &VkCmdRecorder<'a, IGraphics> {
        unsafe {
            self.device.handle.cmd_set_line_width(self.command, width);
        } self
    }

    /// Set the depth bias dynamically.
    fn set_depth_bias(&self, constant_factor: vkfloat, clamp: vkfloat, slope_factor: vkfloat) -> &VkCmdRecorder<'a, IGraphics> {
        unsafe {
            self.device.handle.cmd_set_depth_bias(self.command, constant_factor, clamp, slope_factor)
        } self
    }

    /// Set the blend constants dynamically.
    fn set_blend_constants(&self, constants: [vkfloat; 4]) -> &VkCmdRecorder<'a, IGraphics> {
        unsafe {
            self.device.handle.cmd_set_blend_constants(self.command, &constants);
        } self
    }

    /// Set the depth bound dynamically.
    fn set_depth_bound(&self, min: vkfloat, max: vkfloat) -> &VkCmdRecorder<'a, IGraphics> {
        unsafe {
            self.device.handle.cmd_set_depth_bounds(self.command, min, max);
        } self
    }

    /// Set the stencil compare mask dynamically.
    fn set_stencil_compare_mask(&self, face: vk::StencilFaceFlags, mask: vkuint) -> &VkCmdRecorder<'a, IGraphics> {
        unsafe {
            self.device.handle.cmd_set_stencil_compare_mask(self.command, face, mask);
        } self
    }

    /// Set the stencil write mask dynamically.
    fn set_stencil_write_mask(&self, face: vk::StencilFaceFlags, mask: vkuint) -> &VkCmdRecorder<'a, IGraphics> {
        unsafe {
            self.device.handle.cmd_set_stencil_compare_mask(self.command, face, mask);
        } self
    }

    /// Set the stencil reference dynamically.
    fn set_stencil_reference(&self, face: vk::StencilFaceFlags, reference: vkuint) -> &VkCmdRecorder<'a, IGraphics> {
        unsafe {
            self.device.handle.cmd_set_stencil_reference(self.command, face, reference);
        } self
    }

    fn push_constants(&self, layout: vk::PipelineLayout, stage: vk::ShaderStageFlags, offset: vkuint, data: &[u8]) -> &VkCmdRecorder<'a, IGraphics> {
        unsafe {
            self.device.handle.cmd_push_constants(self.command, layout, stage, offset, data);
        } self
    }

    fn bind_pipeline(&self, pipeline: vk::Pipeline) -> &VkCmdRecorder<'a, IGraphics> {
        unsafe {
            self.device.handle.cmd_bind_pipeline(self.command, IGraphics::BIND_POINT, pipeline);
        } self
    }

    fn bind_vertex_buffers(&self, first_binding: vkuint, buffers: &[vk::Buffer], offsets: &[vkbytes]) -> &VkCmdRecorder<'a, IGraphics> {

        unsafe {
            self.device.handle.cmd_bind_vertex_buffers(self.command, first_binding, buffers, offsets);
        } self
    }

    fn bind_index_buffer(&self, buffer: vk::Buffer, index_type: vk::IndexType, offset: vkbytes) -> &Self {
        unsafe {
            self.device.handle.cmd_bind_index_buffer(self.command, buffer, offset, index_type);
        } self
    }

    fn bind_descriptor_sets(&self, layout: vk::PipelineLayout, first_set: vkuint, descriptor_sets: &[vk::DescriptorSet], dynamic_offsets: &[vkuint]) -> &VkCmdRecorder<'a, IGraphics> {

        unsafe {
            self.device.handle.cmd_bind_descriptor_sets(self.command, IGraphics::BIND_POINT, layout, first_set, descriptor_sets, dynamic_offsets);
        } self
    }

    fn draw(&self, vertex_count: vkuint, instance_count: vkuint, first_vertex: vkuint, first_instance: vkuint) -> &VkCmdRecorder<'a, IGraphics> {
        unsafe {
            self.device.handle.cmd_draw(self.command, vertex_count, instance_count, first_vertex, first_instance);
        } self
    }

    fn draw_indexed(&self, index_count: vkuint, instance_count: vkuint, first_index: vkuint, vertex_offset: vksint, first_instance: vkuint) -> &VkCmdRecorder<'a, IGraphics> {
        unsafe {
            self.device.handle.cmd_draw_indexed(self.command, index_count, instance_count, first_index, vertex_offset, first_instance);
        } self
    }

    fn end_render_pass(&self) -> &VkCmdRecorder<'a, IGraphics> {
        // Ending the render pass will add an implicit barrier transitioning the frame buffer color attachment vk::IMAGE_LAYOUT_PRESENT_SRC_KHR for presenting it to the windowing system.
        unsafe {
            self.device.handle.cmd_end_render_pass(self.command);
        } self
    }
}

pub trait CmdGraphicsApi {

    fn begin_render_pass(&self, bi: RenderPassBI) -> &Self;

    fn set_viewport(&self, first_viewport: vkuint, viewports: &[vk::Viewport]) -> &Self;

    fn set_scissor(&self, first_scissor: vkuint, scissors: &[vk::Rect2D]) -> &Self;

    fn set_line_width(&self, width: vkfloat) -> &Self;

    fn set_depth_bias(&self, constant_factor: vkfloat, clamp: vkfloat, slope_factor: vkfloat) -> &Self;

    fn set_blend_constants(&self, constants: [vkfloat; 4]) -> &Self;

    fn set_depth_bound(&self, min: vkfloat, max: vkfloat) -> &Self;

    fn set_stencil_compare_mask(&self, face: vk::StencilFaceFlags, mask: vkuint) -> &Self;

    fn set_stencil_write_mask(&self, face: vk::StencilFaceFlags, mask: vkuint) -> &Self;

    fn set_stencil_reference(&self, face: vk::StencilFaceFlags, reference: vkuint) -> &Self;

    fn push_constants(&self, layout: vk::PipelineLayout, stage: vk::ShaderStageFlags, offset: vkuint, data: &[u8]) -> &Self;

    fn bind_pipeline(&self, pipeline: vk::Pipeline) -> &Self;

    fn bind_vertex_buffers(&self, first_binding: vkuint, buffers: &[vk::Buffer], offsets: &[vkbytes]) -> &Self;

    fn bind_index_buffer(&self, buffer: vk::Buffer, index_type: vk::IndexType, offset: vkbytes) -> &Self;

    fn bind_descriptor_sets(&self, layout: vk::PipelineLayout, first_set: vkuint, descriptor_sets: &[vk::DescriptorSet], dynamic_offsets: &[vkuint]) -> &Self;

    fn draw(&self, vertex_count: vkuint, instance_count: vkuint, first_vertex: vkuint, first_instance: vkuint) -> &Self;

    fn draw_indexed(&self, index_count: vkuint, instance_count: vkuint, first_index: vkuint, vertex_offset: vksint, first_instance: vkuint) -> &Self;

    fn end_render_pass(&self) -> &Self;
}

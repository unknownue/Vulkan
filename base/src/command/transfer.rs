
use ash::vk;
use ash::version::DeviceV1_0;

use crate::command::VkCommandType;
use crate::command::recorder::VkCmdRecorder;

use crate::ci::image::ImageBarrierCI;

pub struct ITransfer;

impl VkCommandType for ITransfer {
    const BIND_POINT: vk::PipelineBindPoint = vk::PipelineBindPoint::GRAPHICS;
}

impl<'a> CmdTransferApi for VkCmdRecorder<'a, ITransfer> {

    fn copy_buf2buf(&self, src_buffer_handle: vk::Buffer, dst_buffer_handle: vk::Buffer, regions: &[vk::BufferCopy]) -> &Self {
        unsafe {
            self.device.logic.handle.cmd_copy_buffer(self.command, src_buffer_handle, dst_buffer_handle, regions);
        } self
    }

    fn copy_buf2img(&self, src_handle: vk::Buffer, dst_handle: vk::Image, dst_layout: vk::ImageLayout, regions: &[vk::BufferImageCopy]) -> &Self {
        unsafe {
            self.device.logic.handle.cmd_copy_buffer_to_image(self.command, src_handle, dst_handle, dst_layout, regions);
        } self
    }

    fn copy_img2buf(&self, src_handle: vk::Image, src_layout: vk::ImageLayout, dst_buffer: vk::Buffer, regions: &[vk::BufferImageCopy]) -> &Self {
        unsafe {
            self.device.logic.handle.cmd_copy_image_to_buffer(self.command, src_handle, src_layout, dst_buffer, regions);
        } self
    }

    fn copy_img2img(&self,src_handle: vk::Image, src_layout: vk::ImageLayout, dst_handle: vk::Image, dst_layout: vk::ImageLayout, regions: &[vk::ImageCopy]) -> &Self {
        unsafe {
            self.device.logic.handle.cmd_copy_image(self.command, src_handle, src_layout, dst_handle, dst_layout, regions);
        } self
    }

    fn image_pipeline_barrier(&self, src_stage: vk::PipelineStageFlags, dst_stage: vk::PipelineStageFlags, dependencies: vk::DependencyFlags, image_barriers: Vec<ImageBarrierCI>) -> &Self {

        let barriers: Vec<vk::ImageMemoryBarrier> = image_barriers.into_iter()
            .map(|b| b.into()).collect();

        unsafe {
            self.device.logic.handle.cmd_pipeline_barrier(self.command, src_stage, dst_stage, dependencies, &[], &[], &barriers);
        } self
    }

    fn blit_image(&self, src_handle: vk::Image, src_layout: vk::ImageLayout, dst_handle: vk::Image, dst_layout: vk::ImageLayout, regions: &[vk::ImageBlit], filter: vk::Filter) -> &Self {
        unsafe {
            self.device.logic.handle.cmd_blit_image(self.command, src_handle, src_layout, dst_handle, dst_layout, regions, filter);
        } self
    }
}

pub trait CmdTransferApi {

    fn copy_buf2buf(&self, src_buffer_handle: vk::Buffer, dst_buffer_handle: vk::Buffer, regions: &[vk::BufferCopy]) -> &Self;

    fn copy_buf2img(&self, src_handle: vk::Buffer, dst_handle: vk::Image, dst_layout: vk::ImageLayout, regions: &[vk::BufferImageCopy]) -> &Self;

    fn copy_img2buf(&self, src_handle: vk::Image, src_layout: vk::ImageLayout, dst_buffer: vk::Buffer, regions: &[vk::BufferImageCopy]) -> &Self;

    fn copy_img2img(&self,src_handle: vk::Image, src_layout: vk::ImageLayout, dst_handle: vk::Image, dst_layout: vk::ImageLayout, regions: &[vk::ImageCopy]) -> &Self;

    fn image_pipeline_barrier(&self, src_stage: vk::PipelineStageFlags, dst_stage: vk::PipelineStageFlags, dependencies: vk::DependencyFlags, image_barriers: Vec<ImageBarrierCI>) -> &Self;

    fn blit_image(&self, src_handle: vk::Image, src_layout: vk::ImageLayout, dst_handle: vk::Image, dst_layout: vk::ImageLayout, regions: &[vk::ImageBlit], filter: vk::Filter) -> &Self;
}

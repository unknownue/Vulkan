
mod window;
mod loops;


use ash::vk;
use crate::context::VkDevice;
use crate::utils::frame::FrameAction;
use crate::error::VkResult;

pub trait Workflow {

    fn init(&mut self, _device: &VkDevice) -> VkResult<()> {
        Ok(())
    }

    fn render_frame(&mut self, device: &VkDevice, device_available: vk::Fence, image_available: vk::Semaphore, image_index: usize, delta_time: f32) -> VkResult<vk::Semaphore>;

    fn swapchain_reload(&mut self, _device: &VkDevice) -> VkResult<()> {
        Ok(())
    }

    fn receive_input(&mut self, delta_time: f32) -> FrameAction;

    fn deinit(&mut self, device: &VkDevice) -> VkResult<()>;
}


pub use self::window::{WindowContext, WindowConfig};
pub use self::loops::ProcPipeline;

mod window;
mod loops;


use ash::vk;
use crate::context::{VkDevice, VkSwapchain};
use crate::utils::frame::FrameAction;
use crate::input::EventController;
use crate::error::VkResult;

//
// Initialize Vulkan Context
//        ↓             <---------------------------------------↑
//        ↓             ↓      (swapchain_reload if happen)     ↓
//        ↓             ↓                                       ↑(game loop)
//        ↓             ↓                                       ↑
//      init() -------------> receive_input --> render_frame --------> deinit ----------> destroy Vulkan Context.
//                                                                  (terminate program)
pub trait RenderWorkflow {

    fn init(&mut self, _device: &VkDevice) -> VkResult<()> {
        Ok(())
    }

    fn render_frame(&mut self, device: &VkDevice, device_available: vk::Fence, await_present: vk::Semaphore, image_index: usize, delta_time: f32) -> VkResult<vk::Semaphore>;

    fn swapchain_reload(&mut self, _device: &VkDevice, _new_chain: &VkSwapchain) -> VkResult<()> {
        Ok(())
    }

    fn receive_input(&mut self, inputer: &EventController, delta_time: f32) -> FrameAction;

    fn deinit(&mut self, device: &mut VkDevice) -> VkResult<()>;
}

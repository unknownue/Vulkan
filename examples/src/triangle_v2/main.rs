//!
//! Vulkan Example - Basic indexed triangle rendering
//!
//! This example use more wrapper functions to simplify the code.
//!

mod data;
mod example;

const WINDOW_WIDTH : u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;
const WINDOW_TITLE: &'static str = "Vulkan Example - Basic indexed triangle";

fn main() {

    use vkbase::{WindowConfig, WindowContext};
    use vkbase::context::VulkanContext;
    use vkbase::ProcPipeline;

    let mut win_config = WindowConfig::default();
    win_config.dimension.width  = WINDOW_WIDTH;
    win_config.dimension.height = WINDOW_HEIGHT;
    win_config.title = WINDOW_TITLE.to_string();

    let window = WindowContext::new(win_config)
        .expect("Error when creating Window Context");

    let vk_context = VulkanContext::new(&window)
        .build().expect("Error when creating Vulkan Context");

    let app = example::VulkanExample::new(&vk_context)
        .expect("Error when initializing application");

    let mut entry = ProcPipeline::new(window, vk_context).unwrap();

    match entry.launch(app) {
        | Ok(_) => {},
        | Err(e) => {
            eprintln!("{}", e)
        }
    }
}

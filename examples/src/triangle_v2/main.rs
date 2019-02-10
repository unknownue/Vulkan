//!
//! Vulkan Example - Basic indexed triangle rendering
//!
//! This example use more wrapper functions to simplify the code.
//!

mod data;
mod example;
mod helper;

const WINDOW_WIDTH : u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;
const WINDOW_TITLE: &'static str = "Vulkan Example - Basic indexed triangle";

fn main() {

    // TODO: handle unwrap() in some way.

    let mut win_config = vkbase::WindowConfig::default();
    win_config.dimension.width  = WINDOW_WIDTH;
    win_config.dimension.height = WINDOW_HEIGHT;
    win_config.title = WINDOW_TITLE.to_string();

    let window = vkbase::WindowContext::new(win_config).unwrap();

    let vk_context = vkbase::context::VulkanContext::new(&window)
        .build().unwrap();

    let app = example::VulkanExample::new(&vk_context)
        .unwrap();

    let mut entry = vkbase::ProcPipeline::new(window, vk_context).unwrap();

    match entry.launch(app) {
        | Ok(_) => {},
        | Err(e) => {
            eprintln!("{}", e)
        }
    }
}

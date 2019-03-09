//!
//! Vulkan Example - Shader specialization constants
//!
//! Use specialization constants for multiple pipelines.
//! For more details, visit https://www.khronos.org/registry/vulkan/specs/misc/GL_KHR_vulkan_glsl.txt.
//!

mod example;

const WINDOW_WIDTH : u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;
const WINDOW_TITLE: &'static str = "Vulkan Example - Specialization constants";

fn main() {

    use vkbase::{WindowConfig, WindowContext};
    use vkbase::context::VulkanContext;
    use vkbase::ProcPipeline;

    let mut win_config = WindowConfig::default();
    win_config.dimension.width  = WINDOW_WIDTH;
    win_config.dimension.height = WINDOW_HEIGHT;
    win_config.title = WINDOW_TITLE.to_string();
    win_config.is_cursor_hide = true;
    win_config.is_cursor_grap = true;

    let window = WindowContext::new(win_config)
        .expect("Error when creating Window Context");

    let mut vk_context = VulkanContext::new(&window)
        .build().expect("Error when creating Vulkan Context");

    let app = example::VulkanExample::new(&mut vk_context)
        .expect("Error when initializing application");

    let entry = ProcPipeline::new(window, vk_context).unwrap();

    match entry.launch(app) {
        | Ok(_) => {},
        | Err(e) => {
            eprintln!("{}", e)
        }
    }
}

//!
//! Vulkan Example - Pipeline state objects
//!
//! Using different pipelines in one single renderpass.
//!

mod example;

const WINDOW_WIDTH : u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;
const WINDOW_TITLE: &'static str = "Vulkan Example - Pipeline state objects";

fn main() {

    use vkbase::{WindowConfig, WindowContext};
    use vkbase::context::PhysicalDevConfig;
    use vkbase::ProcPipeline;
    use vkbase::context::VulkanContext;

    let mut win_config = WindowConfig::default();
    win_config.dimension.width  = WINDOW_WIDTH;
    win_config.dimension.height = WINDOW_HEIGHT;
    win_config.title = WINDOW_TITLE.to_string();
    win_config.is_cursor_hide = true;
    win_config.is_cursor_grap = true;

    let window = WindowContext::new(win_config)
        .expect("Error when creating Window Context");

    let mut phy_config = PhysicalDevConfig::default();
    phy_config.request_features.fill_mode_non_solid = ash::vk::TRUE;
    // phy_config.request_features.wide_lines = ash::vk::TRUE;

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

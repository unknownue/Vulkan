//!
//! Vulkan Example - Descriptor Sets
//!
//! Using descriptor sets for passing data to shader stages.
//!

mod example;

const WINDOW_WIDTH : u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;
const WINDOW_TITLE: &'static str = "Vulkan Example - Using Descriptor Sets";

fn main() {

    use vkbase::{WindowConfig, WindowContext};
    use vkbase::context::{PhysicalDevConfig, VulkanContext};
    use vkbase::ProcPipeline;

    let mut win_config = WindowConfig::default();
    win_config.dimension.width  = WINDOW_WIDTH;
    win_config.dimension.height = WINDOW_HEIGHT;
    win_config.title = WINDOW_TITLE.to_string();
    win_config.is_cursor_hide = true;
    win_config.is_cursor_grap = true;

    let window = WindowContext::new(win_config)
        .expect("Error when creating Window Context");

    let mut phy_config = PhysicalDevConfig::default();
    phy_config.request_features.sampler_anisotropy = ash::vk::TRUE;

    let mut vk_context = VulkanContext::new(&window)
        .with_physical_device_config(phy_config)
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

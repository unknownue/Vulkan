//!
//! Vulkan Example - Dynamic uniform buffers
//!
//! Instead of using one uniform buffer per-object, this example allocates one big uniform buffer
//! with respect to the alignment reported by the device via minUniformBufferOffsetAlignment that
//! contains all matrices for the objects in the scene.
//!
//! The used descriptor type vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC allows to set a dynamic
//! offset that used to pass data from the single uniform buffer to the connected shader binding point.
//!

mod data;
mod example;

const WINDOW_WIDTH : u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;
const WINDOW_TITLE: &'static str = "Vulkan Example - Dynamic uniform buffers";

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

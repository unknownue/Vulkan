
use ash::vk;

use crate::error::{VkResult, VkError};

// TODO: Add docs for Window Config.

pub struct WindowConfig {

    pub title: String,
    pub mode: WindowMode,

    pub dimension: vk::Extent2D,
    pub max_dimension: Option<vk::Extent2D>,
    pub min_dimension: Option<vk::Extent2D>,

    pub always_on_top: bool,
    pub is_resizable: bool,

    pub is_cursor_grap: bool,
    pub is_cursor_hide: bool,
}

impl Default for WindowConfig {

    fn default() -> WindowConfig {

        WindowConfig {
            title: String::from("Vulkan Example"),
            mode: WindowMode::Normal,

            dimension: vk::Extent2D {
                width : 1280,
                height: 720,
            },
            max_dimension: None,
            min_dimension: None,

            always_on_top: false,
            is_resizable: true,

            is_cursor_grap: true,
            is_cursor_hide: true,
        }
    }
}

pub enum WindowMode {
    Normal,
    Maximized,
    Fullscreen,
}


pub struct WindowContext {

    pub(crate) event_loop: winit::EventsLoop,
    pub(crate) handle: winit::Window,
}

impl WindowContext {

    pub fn new(config: WindowConfig) -> VkResult<WindowContext> {

        let event_loop = winit::EventsLoop::new();

        let mut builder = winit::WindowBuilder::new()
            .with_title(config.title)
            .with_dimensions((config.dimension.width, config.dimension.height).into())
            .with_always_on_top(config.always_on_top)
            .with_resizable(config.is_resizable);

        if let Some(min) = config.min_dimension {
            builder = builder.with_min_dimensions((min.width, min.height).into());
        }

        if let Some(max) = config.max_dimension {
            builder = builder.with_max_dimensions((max.width, max.height).into());
        }

        builder = match config.mode {
            | WindowMode::Maximized => {
                builder.with_maximized(true)
            },
            | WindowMode::Fullscreen => {
                let primary_monitor = event_loop.get_primary_monitor();
                builder.with_fullscreen(Some(primary_monitor))
            },
            | WindowMode::Normal => {
                builder
            },
        };

        let window = WindowContext {
            handle: builder.build(&event_loop)
                .map_err(|e| VkError::window(e.to_string()))?,
            event_loop,
        };
        Ok(window)
    }

    pub fn dimension(&self) -> VkResult<vk::Extent2D> {

        self.handle.get_inner_size()
            .and_then(|dim| Some(ash::vk::Extent2D { width : dim.width as _, height: dim.height as _, }))
            .ok_or(VkError::window("Failed to get dimension of current window."))
    }
}

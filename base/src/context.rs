
pub use self::device::VkDevice;
pub use self::swapchain::SwapchainSyncError;

mod instance;
mod debug;
mod surface;
mod device;
mod swapchain;


use ash::version::DeviceV1_0;
use crate::error::{VkResult, VkError};

#[derive(Default)]
pub struct VulkanConfig {

    instance  : instance::InstanceConfig,
    debugger  : debug::ValidationConfig,
    dev_logic : device::LogicDevConfig,
    dev_phy   : device::PhysicalDevConfig,
    swapchain : swapchain::SwapchainConfig,
}

pub struct VulkanContext {

    instance  : instance::VkInstance,
    debugger  : debug::VkDebugger,
    surface   : surface::VkSurface,

    pub(super) swapchain: swapchain::VkSwapchain,
    pub device: device::VkDevice,
}

impl VulkanContext {

    pub fn new(window: &winit::Window) -> VulkanContextBuilder {

        VulkanContextBuilder {
            window,
            config: VulkanConfig::default(),
        }
    }

    pub fn recreate_swapchain(&mut self, window: &winit::Window) -> VkResult<()> {

        let dimension = window_dimension(window)?;
        self.swapchain.rebuild(&self.instance, &self.device, &self.surface, dimension)?;

        Ok(())
    }

    pub fn wait_idle(&self) -> VkResult<()> {
        unsafe {
            self.device.logic.handle.device_wait_idle()
                .map_err(|_| VkError::device("Device Waiting Idle"))?;
        }

        Ok(())
    }

    pub fn discard(&self) {

        self.swapchain.discard(&self.device);
        self.device.logic.discard();
        self.debugger.discard();
        self.instance.discard();
    }
}

pub struct VulkanContextBuilder<'a> {

    window: &'a winit::Window,
    config: VulkanConfig,
}

impl<'a> VulkanContextBuilder<'a> {

    pub fn with_instance_config(mut self, config: instance::InstanceConfig) -> VulkanContextBuilder<'a> {
        self.config.instance = config;
        self
    }

    pub fn with_debugger_config(mut self, config: debug::ValidationConfig) -> VulkanContextBuilder<'a> {
        self.config.debugger = config;
        self
    }

    pub fn with_logic_device_config(mut self, config: device::LogicDevConfig) -> VulkanContextBuilder<'a> {
        self.config.dev_logic = config;
        self
    }

    pub fn with_physical_device_config(mut self, config: device::PhysicalDevConfig) -> VulkanContextBuilder<'a> {
        self.config.dev_phy = config;
        self
    }

    pub fn with_swapchain_config(mut self, config: swapchain::SwapchainConfig) -> VulkanContextBuilder<'a> {
        self.config.swapchain = config;
        self
    }

    pub fn build(self) -> VkResult<VulkanContext> {

        let instance = instance::VkInstance::new(self.config.instance)?;
        let debugger = debug::VkDebugger::new(&instance, self.config.debugger)?;
        let surface = surface::VkSurface::new(&instance, &self.window)?;

        let phy_device = device::VkPhysicalDevice::new(&instance, self.config.dev_phy)?;
        let logic_device = device::VkLogicalDevice::new(&instance, &phy_device, self.config.dev_logic)?;
        let device = device::VkDevice {
            phy: phy_device,
            logic: logic_device,
        };

        let dimension = window_dimension(&self.window)?;
        let swapchain = swapchain::VkSwapchain::new(&instance, &device, &surface, self.config.swapchain, dimension)?;

        let context = VulkanContext { instance, debugger, surface, device, swapchain };
        Ok(context)
    }
}


fn window_dimension(window: &winit::Window) -> VkResult<ash::vk::Extent2D> {
    window.get_inner_size()
        .and_then(|dim| Some(ash::vk::Extent2D { width : dim.width as _, height: dim.height as _, }))
        .ok_or(VkError::window("Failed to get dimension of current window."))
}

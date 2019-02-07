
mod instance;
mod debug;
mod surface;
mod device;
mod swapchain;


use crate::error::VkResult;

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
    device    : device::VkDevice,
    swapchain : swapchain::VkSwapchain,
}

impl VulkanContext {

    pub fn new(window: &winit::Window) -> VulkanContextBuilder {

        VulkanContextBuilder {
            window,
            config: VulkanConfig::default(),
        }
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

        let swapchain = swapchain::VkSwapchain::new(&instance, &device, &surface, self.config.swapchain, None)?;

        let context = VulkanContext { instance, debugger, surface, device, swapchain };
        Ok(context)
    }
}

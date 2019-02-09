
pub use self::device::VkDevice;
pub use self::swapchain::SwapchainSyncError;

pub use self::instance::InstanceConfig;
pub use self::debug::ValidationConfig;
pub use self::device::{LogicDevConfig, PhysicalDevConfig};
pub use self::swapchain::SwapchainConfig;

mod instance;
mod debug;
mod surface;
mod device;
mod swapchain;


use ash::version::DeviceV1_0;
use crate::workflow::WindowContext;
use crate::error::{VkResult, VkError};

#[derive(Default)]
pub struct VulkanConfig {

    instance  : InstanceConfig,
    debugger  : ValidationConfig,
    dev_logic : LogicDevConfig,
    dev_phy   : PhysicalDevConfig,
    swapchain : SwapchainConfig,
}

pub struct VulkanContext {

    instance  : instance::VkInstance,
    debugger  : debug::VkDebugger,
    surface   : surface::VkSurface,

    pub(super) swapchain: swapchain::VkSwapchain,
    pub(crate) device: device::VkDevice,
}

impl VulkanContext {

    pub fn new(window: &WindowContext) -> VulkanContextBuilder {

        VulkanContextBuilder {
            window,
            config: VulkanConfig::default(),
        }
    }

    pub(super) fn recreate_swapchain(&mut self, window: &WindowContext) -> VkResult<()> {

        let dimension = window.dimension()?;
        self.swapchain.rebuild(&self.instance, &self.device, &self.surface, dimension)?;

        Ok(())
    }

    pub(super) fn wait_idle(&self) -> VkResult<()> {
        unsafe {
            self.device.logic.handle.device_wait_idle()
                .map_err(|_| VkError::device("Device Waiting Idle"))?;
        }

        Ok(())
    }

    pub(super) fn discard(&self) {

        self.swapchain.discard(&self.device);
        self.device.logic.discard();
        self.surface.discard();
        self.debugger.discard();
        self.instance.discard();
    }
}

pub struct VulkanContextBuilder<'a> {

    window: &'a WindowContext,
    config: VulkanConfig,
}

impl<'a> VulkanContextBuilder<'a> {

    pub fn with_instance_config(mut self, config: InstanceConfig) -> VulkanContextBuilder<'a> {
        self.config.instance = config;
        self
    }

    pub fn with_debugger_config(mut self, config: ValidationConfig) -> VulkanContextBuilder<'a> {
        self.config.debugger = config;
        self
    }

    pub fn with_logic_device_config(mut self, config: LogicDevConfig) -> VulkanContextBuilder<'a> {
        self.config.dev_logic = config;
        self
    }

    pub fn with_physical_device_config(mut self, config: PhysicalDevConfig) -> VulkanContextBuilder<'a> {
        self.config.dev_phy = config;
        self
    }

    pub fn with_swapchain_config(mut self, config: SwapchainConfig) -> VulkanContextBuilder<'a> {
        self.config.swapchain = config;
        self
    }

    pub fn build(self) -> VkResult<VulkanContext> {

        let instance = instance::VkInstance::new(self.config.instance)?;
        let debugger = debug::VkDebugger::new(&instance, self.config.debugger)?;
        let surface = surface::VkSurface::new(&instance, &self.window.handle)?;

        let phy_device = device::VkPhysicalDevice::new(&instance, self.config.dev_phy)?;
        let logic_device = device::VkLogicalDevice::new(&instance, &phy_device, self.config.dev_logic)?;
        let device = device::VkDevice {
            phy: phy_device,
            logic: logic_device,
        };

        let dimension = self.window.dimension()?;
        let swapchain = swapchain::VkSwapchain::new(&instance, &device, &surface, self.config.swapchain, dimension)?;

        let context = VulkanContext { instance, debugger, surface, device, swapchain };
        Ok(context)
    }
}

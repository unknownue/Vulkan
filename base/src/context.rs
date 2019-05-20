
pub use self::device::{VkDevice, VkLogicalDevice, VkPhysicalDevice};
pub use self::device::{VkObjectDiscardable, VkObjectAllocatable, VkObjectBindable};
pub use self::device::VmaResourceDiscardable;
pub use self::device::{VkObjectWaitable, VkSubmitCI};
pub use self::swapchain::{VkSwapchain, SwapchainSyncError};

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
use crate::error::{VkResult, VkError, VkErrorKind};

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

    pub swapchain: swapchain::VkSwapchain,
    pub device: device::VkDevice,
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

    pub(super) fn discard(self) {

        self.swapchain.discard(&self.device);
        drop(self.swapchain);

        self.device.drop_self();

        drop(self.surface);
        drop(self.debugger);
        drop(self.instance);
    }
}

pub struct VulkanContextBuilder<'a> {

    window: &'a WindowContext,
    config: VulkanConfig,
}

impl<'a> VulkanContextBuilder<'a> {

    pub fn with_instance_config(mut self, config: InstanceConfig) -> VulkanContextBuilder<'a> {
        self.config.instance = config; self
    }

    pub fn with_debugger_config(mut self, config: ValidationConfig) -> VulkanContextBuilder<'a> {
        self.config.debugger = config; self
    }

    pub fn with_logic_device_config(mut self, config: LogicDevConfig) -> VulkanContextBuilder<'a> {
        self.config.dev_logic = config; self
    }

    pub fn with_physical_device_config(mut self, config: PhysicalDevConfig) -> VulkanContextBuilder<'a> {
        self.config.dev_phy = config; self
    }

    pub fn with_swapchain_config(mut self, config: SwapchainConfig) -> VulkanContextBuilder<'a> {
        self.config.swapchain = config; self
    }

    pub fn build(self) -> VkResult<VulkanContext> {

        let instance = instance::VkInstance::new(self.config.instance, &self.config.debugger)?;
        let debugger = debug::VkDebugger::new(&instance, self.config.debugger)?;
        let surface = surface::VkSurface::new(&instance, &self.window.handle)?;

        let phy_device = device::VkPhysicalDevice::new(&instance, self.config.dev_phy)?;
        let logic_device = device::VkLogicalDevice::new(&instance, &phy_device, self.config.dev_logic)?;
        let vma = VulkanContextBuilder::build_vma(&instance, &phy_device, &logic_device)?;
        let device = device::VkDevice::new(logic_device, phy_device, vma)?;

        let dimension = self.window.dimension()?;
        let swapchain = swapchain::VkSwapchain::new(&instance, &device, &surface, self.config.swapchain, dimension)?;

        let context = VulkanContext { instance, debugger, surface, device, swapchain };
        Ok(context)
    }

    /// Create Vulkan Memory Allocator object for VkDevice.
    fn build_vma(instance: &instance::VkInstance, phy_device: &VkPhysicalDevice, logic_device: &VkLogicalDevice) -> VkResult<vma::Allocator> {

        let allocator_ci = vma::AllocatorCreateInfo {
            physical_device: phy_device.handle,
            device: logic_device.handle.clone(),
            instance: instance.handle.clone(),
            // handle synchronization by myself.
            flags: vma::AllocatorCreateFlags::EXTERNALLY_SYNCHRONIZED,
            // tell vma use default block size.
            preferred_large_heap_block_size: 0,
            // this crate does not use `lost allocations` feature, so this field does not matter.
            frame_in_use_count: 0,
            // disable limitation on memory heap.
            heap_size_limits: None,
        };

        let allocator = vma::Allocator::new(&allocator_ci)
            .map_err(VkErrorKind::Vma)?;
        Ok(allocator)
    }
}

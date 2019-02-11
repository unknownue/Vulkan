
use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::VkDevice;


impl crate::context::VulkanObject for vk::DeviceMemory {

    fn discard(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.free_memory(self, None);
        }
    }
}

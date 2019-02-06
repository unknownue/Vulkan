
use ash::vk;
use ash::version::InstanceV1_0;

use crate::context::instance::VkInstance;
use crate::context::physical::{PhysicalDevConfig, PhyDeviceTmp};

// Physical Property -----------------------------------------------------------------
pub(super) fn query_device_property(instance: &VkInstance, phy_device: vk::PhysicalDevice, config: &PhysicalDevConfig) -> PhyDeviceTmp {

    let device_property = unsafe {
        instance.handle.get_physical_device_properties(phy_device)
    };

    PhyDeviceTmp {
        handle: phy_device,
        property: device_property,
    }
}

pub(super) fn print_device_properties(property: &vk::PhysicalDeviceProperties) {

    use crate::utils::cast::chars2string;

    let device_name = chars2string(&property.device_name);
    println!("[Info] Using device: {}", &device_name);

    use ash::{vk_version_major, vk_version_minor, vk_version_patch};
    let (major, minor, patch) = (
        vk_version_major!(property.api_version),
        vk_version_minor!(property.api_version),
        vk_version_patch!(property.api_version),
    );
    println!("[Info] Device API version: {}.{}.{}", major, minor, patch);

    let device_type = match property.device_type {
        | vk::PhysicalDeviceType::CPU            => "CPU",
        | vk::PhysicalDeviceType::INTEGRATED_GPU => "Integrated GPU",
        | vk::PhysicalDeviceType::DISCRETE_GPU   => "Discrete GPU",
        | vk::PhysicalDeviceType::VIRTUAL_GPU    => "Virtual GPU",
        | _ => "Unknown",
    };
    println!("[Info] Device Type: {}", device_type);
}
// -----------------------------------------------------------------------------------

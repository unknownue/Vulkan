
mod property;
mod extension;
mod feature;

use ash::vk;
use ash::version::InstanceV1_0;

use crate::context::instance::VkInstance;
use crate::error::{VkResult, VkError};
use crate::vkuint;

use std::ffi::CString;


pub struct PhysicalDevConfig {

    print_device_properties: bool,
    device_type_preference: vk::PhysicalDeviceType,

    print_available_extensions: bool,
    request_extensions: Vec<CString>,

    print_available_features: bool,
    request_features: vk::PhysicalDeviceFeatures,
}

impl Default for PhysicalDevConfig {

    fn default() -> PhysicalDevConfig {

        PhysicalDevConfig {
            print_device_properties: false,
            device_type_preference: vk::PhysicalDeviceType::DISCRETE_GPU,

            print_available_extensions: false,
            request_extensions: vec![
                extension::DeviceExtensionType::Swapchain.name(),
            ],

            print_available_features: false,
            request_features: vk::PhysicalDeviceFeatures::default(),
        }
    }
}

pub struct VkPhysicalDevice {

    handle: vk::PhysicalDevice,
    config: PhysicalDevConfig,

    memories: vk::PhysicalDeviceMemoryProperties,
    families: Vec<vk::QueueFamilyProperties>,
}

impl VkPhysicalDevice {

    pub fn new(instance: &VkInstance, config: PhysicalDevConfig) -> VkResult<VkPhysicalDevice> {

        let alternative_devices = VkPhysicalDevice::query_phy_devices(instance, &config)?;

        let mut selected_device = None;

        for phy_device in alternative_devices.into_iter() {

            // make sure all requested extensions are support by device.
            if extension::is_all_extension_support(instance, &phy_device, &config)? == false {
                continue
            }

            // make sure all requested features are support by device.
            if feature::is_all_features_support(instance, &phy_device, &config) == false {
                continue
            }

            if config.print_device_properties {
                property::print_device_properties(&phy_device.property);
            }

            selected_device = Some(phy_device);
        }

        if let Some(phy_device) = selected_device {

            // get memory properties.
            let memories = unsafe {
                instance.handle.get_physical_device_memory_properties(phy_device.handle)
            };

            let families = unsafe {
                instance.handle.get_physical_device_queue_family_properties(phy_device.handle)
            };

            let dst_device = VkPhysicalDevice {
                handle: phy_device.handle,
                config, memories, families,
            };

            Ok(dst_device)
        } else {

            Err(VkError::other("Failed to find supportive Vulkan device."))
        }
    }

    fn query_phy_devices(instance: &VkInstance, config: &PhysicalDevConfig) -> VkResult<Vec<PhyDeviceTmp>> {

        let alternative_devices = unsafe {
            instance.handle.enumerate_physical_devices()
                .or(Err(VkError::query("Physical Device")))?
        };

        let mut alternative_devices: Vec<PhyDeviceTmp> = alternative_devices.into_iter().map(|phy_device| {
            property::query_device_property(instance, phy_device, &config)
        }).collect();

        // sort available device by their device type.
        alternative_devices.sort_by(|dev1, dev2| {
            use std::cmp::Ordering;

            if dev1.property.device_type == config.device_type_preference {
                Ordering::Less
            } else if dev2.property.device_type == config.device_type_preference {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });

        Ok(alternative_devices)
    }
}

struct PhyDeviceTmp {

    handle: vk::PhysicalDevice,
    property: vk::PhysicalDeviceProperties,
}

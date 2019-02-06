
use ash::vk;
use ash::version::InstanceV1_0;

use crate::context::instance::VkInstance;
use crate::context::physical::{PhysicalDevConfig, PhyDeviceTmp};
use crate::utils::cast::{chars2cstring, chars2string};
use crate::error::{VkResult, VkError};

use std::ffi::CString;

// Physical Extension ----------------------------------------------------------------
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DeviceExtensionType {
    Swapchain,
}

impl DeviceExtensionType {

    pub fn name(&self) -> CString {
        match self {
            | DeviceExtensionType::Swapchain => {
                CString::new("VK_KHR_swapchain").unwrap()
            },
        }
    }
}

pub(super) fn is_all_extension_support(instance: &VkInstance, phy_device: &PhyDeviceTmp, config: &PhysicalDevConfig) -> VkResult<bool> {

    let query_extensions = unsafe {
        instance.handle.enumerate_device_extension_properties(phy_device.handle)
            .or(Err(VkError::query("Device Extensions")))?
    };

    let available_extensions: Vec<CString> = query_extensions.into_iter().map(|extension| {
        chars2cstring(&extension.extension_name)
    }).collect();

    // print available extensions to console if need.
    if config.print_available_extensions {

        println!("[Info] available extensions for {}:", &chars2string(&phy_device.property.device_name));

        available_extensions.iter().for_each(|extension| {
            println!("\t{:?}", extension)
        });
    }

    let result = config.request_extensions.iter().all(|request_extension| {
        available_extensions.contains(request_extension)
    });
    Ok(result)
}
// -----------------------------------------------------------------------------------

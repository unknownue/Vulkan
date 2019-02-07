
use ash::vk;

use crate::context::instance::VkInstance;
use crate::{vklint, vksint, vkchar, vkptr, vkbool};
use crate::error::{VkResult, VkError};

use std::ffi::CStr;
use std::ptr;

#[derive(Debug, Default)]
pub struct ValidationConfig {

    /// `is_enable` tell if validation layer should be enabled.
    pub debug_type: DebugType,
    /// `report_config` specifies the configuration parameters used in Debug Report.
    pub report_config: DebugReportConfig,
    /// `utils_config` specifies the configuration parameters used in Debug Utils.
    pub  utils_config: DebugUtilsConfig,
}

#[derive(Debug, Clone, Copy)]
pub enum DebugType {
    DebugReport,
    DebugUtils,
    None, // set None to disable Debug tools.
}

impl Default for DebugType {

    fn default() -> DebugType {
        DebugType::None
    }
}

/// `DebugInstance` is used as a trait object.
trait DebugInstance {
    /// Destroy this validation tool.
    unsafe fn discard(&self);
}

/// Wrapper class for the validation tools used in Vulkan.
pub struct VkDebugger {

    target: Option<Box<dyn DebugInstance>>,
}

impl VkDebugger {

    pub fn new(instance: &VkInstance, config: ValidationConfig) -> VkResult<VkDebugger> {

        let debugger = match config.debug_type {
            | DebugType::DebugReport => {
                let report = VkDebugReport::new(instance, &config.report_config)?;
                Some(Box::new(report) as Box<dyn DebugInstance>)
            },
            | DebugType::DebugUtils => {
                let utils = VkDebugUtils::new(instance, &config.utils_config)?;
                Some(Box::new(utils) as Box<dyn DebugInstance>)
            },
            | DebugType::None => {
                None
            },
        };

        let result = VkDebugger { target: debugger };
        Ok(result)
    }

    pub fn discard(&self) {

        if let Some(ref debugger) = self.target {
            unsafe {
                debugger.discard();
            }
        }
    }
}


// Debug Report -----------------------------------------------------------------------------------

/// the callback function used in Debug Report.
unsafe extern "system" fn vulkan_debug_report_callback(
    _flags       : vk::DebugReportFlagsEXT,
    _obj_type    : vk::DebugReportObjectTypeEXT,
    _obj         : vklint,
    _location    : usize,
    _code        : vksint,
    _layer_prefix: *const vkchar,
    p_message    : *const vkchar,
    _user_data   : vkptr
) -> u32 {

    println!("[Debug] {:?}", CStr::from_ptr(p_message));
    vk::FALSE
}

/// The configuration parameters used in the initialization of `vk::DebugReport`.
#[derive(Debug)]
pub struct DebugReportConfig {
    /// the message type that Validation Layer would report for.
    pub flags: vk::DebugReportFlagsEXT,
}

impl Default for DebugReportConfig {

    fn default() -> DebugReportConfig {
        DebugReportConfig {
            flags:
                vk::DebugReportFlagsEXT::DEBUG |
                vk::DebugReportFlagsEXT::ERROR |
                // vk::DebugReportFlagsEXT::INFORMATION |
                // vk::DebugReportFlagsEXT::PERFORMANCE_WARNING |
                vk::DebugReportFlagsEXT::WARNING,
        }
    }
}

struct VkDebugReport {
    /// the handle of `vk::DebugReport` object.
    loader: ash::extensions::ext::DebugReport,
    /// the handle of callback function used in Validation Layer.
    callback: vk::DebugReportCallbackEXT,
}

impl VkDebugReport {

    /// Initialize debug extension loader and `vk::DebugReport` object.
    pub fn new(instance: &VkInstance, config: &DebugReportConfig) -> VkResult<VkDebugReport> {

        // load the debug extension.
        let loader = ash::extensions::ext::DebugReport::new(&instance.entry, &instance.handle);

        // configure debug callback.
        let debug_callback_ci = vk::DebugReportCallbackCreateInfoEXT {
            s_type      : vk::StructureType::DEBUG_REPORT_CALLBACK_CREATE_INFO_EXT,
            p_next      : ptr::null(),
            // Enum DebugReportFlags enumerate all available flags.
            flags       : config.flags,
            pfn_callback: Some(vulkan_debug_report_callback),
            p_user_data : ptr::null_mut(),
        };

        let callback = unsafe {
            loader.create_debug_report_callback(&debug_callback_ci, None)
                .or(Err(VkError::create("Debug Report Callback")))?
        };

        let report = VkDebugReport { loader, callback };
        Ok(report)
    }
}

impl DebugInstance for VkDebugReport {

    /// Destroy the `vk::DebugReport` object.
    unsafe fn discard(&self) {
        self.loader.destroy_debug_report_callback(self.callback, None);
    }
}
// ------------------------------------------------------------------------------------------------

// Debug Utils ------------------------------------------------------------------------------------

/// the callback function used in Debug Utils.
unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity : vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type     : vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data  : *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data     : vkptr
) -> vkbool {

    let severity = match message_severity {
        | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "[Verbose]",
        | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => "[Warning]",
        | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR   => "[Error]",
        | vk::DebugUtilsMessageSeverityFlagsEXT::INFO    => "[Info]",
        | _ => "[Unknown]",
    };
    let types = match message_type {
        | vk::DebugUtilsMessageTypeFlagsEXT::GENERAL     => "[General]",
        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION  => "[Validation]",
        | _ => "[Unknown]",
    };
    let message = CStr::from_ptr((*p_callback_data).p_message);
    println!("[Debug]{}{}{:?}", severity, types, message);

    vk::FALSE
}

/// The configuration parameters used in the initialization of `vk::DebugUtils`.
#[derive(Debug)]
pub struct DebugUtilsConfig {

    pub flags    : vk::DebugUtilsMessengerCreateFlagsEXT,
    pub severity : vk::DebugUtilsMessageSeverityFlagsEXT,
    pub types    : vk::DebugUtilsMessageTypeFlagsEXT,
}

impl Default for DebugUtilsConfig {

    fn default() -> DebugUtilsConfig {
        DebugUtilsConfig {
            flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
            severity:
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
                // vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE |
                // vk::DebugUtilsMessageSeverityFlagsEXT::INFO |
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            types:
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL |
                vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE |
                vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        }
    }
}

/// Wrapper class for `vk::DebugUtils` object.
struct VkDebugUtils {
    /// the handle of `vk::DebugUtils` object.
    loader: ash::extensions::ext::DebugUtils,
    /// the handle of callback function used in Validation Layer.
    utils_messenger: vk::DebugUtilsMessengerEXT,
}

impl VkDebugUtils {

    /// Initialize debug report extension loader and `vk::DebugUtilsMessengerExt` object.
    pub fn new(instance: &VkInstance, config: &DebugUtilsConfig) -> VkResult<VkDebugUtils> {

        let loader = ash::extensions::ext::DebugUtils::new(&instance.entry, &instance.handle);

        let messenger_ci = vk::DebugUtilsMessengerCreateInfoEXT {
            s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
            p_next: ptr::null(),
            flags            : config.flags,
            message_severity : config.severity,
            message_type     : config.types,
            pfn_user_callback: Some(vulkan_debug_utils_callback),
            p_user_data      : ptr::null_mut(),
        };

        let utils_messenger = unsafe {
            loader.create_debug_utils_messenger(&messenger_ci, None)
                .or(Err(VkError::create("Debug Utils Callback")))?
        };

        let utils = VkDebugUtils { loader, utils_messenger };
        Ok(utils)
    }
}

impl DebugInstance for VkDebugUtils {

    /// Destroy the `vk::DebugUtils` object.
    unsafe fn discard(&self) {
        self.loader.destroy_debug_utils_messenger(self.utils_messenger, None);
    }
}
// ------------------------------------------------------------------------------------------------

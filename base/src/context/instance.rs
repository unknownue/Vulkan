
use ash::vk;
use ash::vk_make_version;
use ash::version::{InstanceV1_0, EntryV1_0};

use crate::context::debug::DebugType;
use crate::vkuint;
use crate::error::{VkResult, VkError};

use std::ffi::CString;
use std::ptr;

/// The configuration parameters used in the initialization of `vk::Instance`.
pub struct InstanceConfig {

    /// `api_version` must be the highest version of Vulkan that the application is designed to use.
    ///
    /// The patch version number is ignored and only the major and minor versions must match those requested in `api_version`.
    pub api_version: vkuint,
    /// `application_version` is an unsigned integer variable containing the developer-supplied version number of the application.
    pub application_version: vkuint,
    /// `engine_version`is an unsigned integer variable containing the developer-supplied version number of the engine used to create the application.
    pub engine_version: vkuint,
    /// `application_name` is a string containing the name of the application or None if it is not provided.
    pub application_name: String,
    /// `engine_name` is the name of the engine used to create the application or None if it is not provided.
    pub engine_name: String,
    /// `print_available_layers` specific program to print all available instance layers to console.
    pub print_available_layers: bool,
    /// `require_layer_names` specific which layers to load by vulkan.
    pub require_layer_names: Vec<String>,
    /// `debug` specifies the debug tools used in vulkan backend.
    pub debug: DebugType,
}

impl Default for InstanceConfig {

    fn default() -> InstanceConfig {
       InstanceConfig {
           api_version         : vk_make_version!(1, 0, 0),
           application_version : vk_make_version!(1, 0, 0),
           engine_version      : vk_make_version!(1, 0, 97),
           application_name    : String::from("Vulkan Application"),
           engine_name         : String::from("Engine powered by Vulkan"),
           print_available_layers: false,
           require_layer_names : vec![
               // request validation layer by default.
               String::from("VK_LAYER_LUNARG_standard_validation"),
           ],
           debug: DebugType::DebugUtils, // default to use Debug Utils extension.
       }
    }
}

/// Wrapper class for `vk::Instance` object.
pub struct VkInstance {

    /// handle of `vk::Instance`.
    pub(crate) handle: ash::Instance,
    /// the object used in instance creation define in ash crate.
    pub(crate) entry: ash::Entry,
    /// an array to store the names of vulkan layers enabled in instance creation.
    pub(crate) enable_layer_names: Vec<CString>,
}

impl VkInstance {

    /// Initialize `vk::Instance` object.
    pub fn new(config: InstanceConfig) -> VkResult<VkInstance> {

        let entry = ash::Entry::new()
            .or(Err(VkError::unlink("Entry")))?;

        let app_name = CString::new(config.application_name.as_bytes())
            .map_err(|_| VkError::other("Failed to cast application name to CString."))?;
        let engine_name = CString::new(config.engine_name.as_bytes())
            .map_err(|_| VkError::other("Failed to cast engine name to CString."))?;

        let application_info = vk::ApplicationInfo {
            s_type              : vk::StructureType::APPLICATION_INFO,
            p_next              : ptr::null(),
            p_application_name  : app_name.as_ptr(),
            application_version : config.application_version,
            p_engine_name       : engine_name.as_ptr(),
            engine_version      : config.engine_version,
            api_version         : config.api_version,
        };

        // check if all instance layer is support.
        if is_all_instance_layer_support(&entry, config.print_available_layers, &config.require_layer_names)? == false {
            return Err(VkError::unsupported("Some of Vulkan instance layer"))
        }

        // get the names of required vulkan layers.
        let enable_layer_names = layer_names_to_cstring(&config.require_layer_names)?;
        let enable_layer_names_ptr = crate::utils::cast::cstrings2ptrs(&enable_layer_names);
        // get the names of required vulkan extensions.
        let enable_extension_names = VkInstance::require_extensions(config.debug);

        let instance_ci = vk::InstanceCreateInfo {
            s_type                     : vk::StructureType::INSTANCE_CREATE_INFO,
            p_next                     : ptr::null(),
            flags                      : vk::InstanceCreateFlags::empty(),
            p_application_info         : &application_info,
            enabled_layer_count        : enable_layer_names_ptr.len() as _,
            pp_enabled_layer_names     : enable_layer_names_ptr.as_ptr(),
            enabled_extension_count    : enable_extension_names.len() as _,
            pp_enabled_extension_names : enable_extension_names.as_ptr(),
        };

        // create vk::Instance object.
        let handle = unsafe {
            entry.create_instance(&instance_ci, None)
                .or(Err(VkError::unlink("Instance")))?
        };

        let instance = VkInstance { entry, handle, enable_layer_names };
        Ok(instance)
    }

    /// Specify the necessary extensions.
    fn require_extensions(debug: DebugType) -> Vec<*const i8>  {

        // request extension about platform specific surface and debug tools.
        let mut instance_extensions = vec![
            ash::extensions::khr::Surface::name(),
            crate::platforms::platform_surface_names(),
        ];

        match debug {
            | DebugType::DebugReport => instance_extensions.push(ash::extensions::ext::DebugReport::name()),
            | DebugType::DebugUtils  => instance_extensions.push(ash::extensions::ext::DebugUtils::name()),
            | DebugType::None => {},
        }

        instance_extensions.into_iter().map(|extension| {
            extension.as_ptr()
        }).collect()
    }

    /// Destroy the `vk::Instance` object. This function must be called before this wrapper class is dropped.
    ///
    /// Be careful about the destruction order of Vulkan object, and we have better to destroy them manually.
    ///
    /// In Vulkan, all child objects created using instance must have been destroyed prior to destroying instance.
    fn discard(&self) {

        unsafe {
            self.handle.destroy_instance(None);
        }
    }
}

fn is_all_instance_layer_support(entry: &ash::Entry, print_available_layers: bool, required_layers: &[String]) -> VkResult<bool> {

    use crate::utils::cast::chars2string;

    let layer_properties = entry.enumerate_instance_layer_properties()
        .or(Err(VkError::query("Layer Properties")))?;

    let available_layer_names: Vec<String> = layer_properties.into_iter().map(|available_layer| {
        chars2string(&available_layer.layer_name)
    }).collect();

    if print_available_layers {
        println!("[Info] Available instance layers: ");
        available_layer_names.iter().for_each(|layer| {
            println!("\t{}", layer)
        });
    }

    let result = required_layers.iter().all(|required_layer_name| {

        available_layer_names.iter().any(|available_layer| {

            (*available_layer) == (*required_layer_name)
        })
    });

    Ok(result)
}

fn layer_names_to_cstring(layers: &[String]) -> VkResult<Vec<CString>> {

    let mut layer_names = Vec::with_capacity(layers.len());

    for layer in layers.iter() {
        let name_converted = CString::new(layer.as_bytes())
            .map_err(|_| VkError::other("Failed to cast instance layer name to CString."))?;
        layer_names.push(name_converted);
    }

    Ok(layer_names)
}

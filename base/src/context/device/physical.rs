
use ash::vk;
use ash::version::InstanceV1_0;

use crate::context::instance::VkInstance;
use crate::utils::cast::{chars2string, chars2cstring};
use crate::error::{VkResult, VkError};

use std::ffi::CString;


#[derive(Debug, Clone)]
pub struct PhysicalDevConfig {

    pub print_device_properties: bool,
    pub device_type_preference: vk::PhysicalDeviceType,

    pub print_available_extensions: bool,
    pub request_extensions: Vec<CString>,

    pub print_available_features: bool,
    pub request_features: vk::PhysicalDeviceFeatures,
}

impl Default for PhysicalDevConfig {

    fn default() -> PhysicalDevConfig {

        PhysicalDevConfig {
            print_device_properties: false,
            device_type_preference: vk::PhysicalDeviceType::DISCRETE_GPU,

            print_available_extensions: false,
            request_extensions: vec![
                DeviceExtensionType::Swapchain.name(),
            ],

            print_available_features: false,
            request_features: vk::PhysicalDeviceFeatures::default(),
        }
    }
}

pub struct VkPhysicalDevice {

    pub device_name: String,
    pub handle: vk::PhysicalDevice,
    pub memories: vk::PhysicalDeviceMemoryProperties,
    pub depth_format: vk::Format,

    pub limits: vk::PhysicalDeviceLimits,

    features_enable: vk::PhysicalDeviceFeatures,
    config: PhysicalDevConfig,
}

impl VkPhysicalDevice {

    pub(crate) fn new(instance: &VkInstance, config: PhysicalDevConfig) -> VkResult<VkPhysicalDevice> {

        let alternative_devices = VkPhysicalDevice::query_phy_devices(instance, &config)?;

        let mut selected_device = None;

        for phy_device in alternative_devices.into_iter() {

            // make sure all requested extensions are support by device.
            if is_all_extension_support(instance, &phy_device, &config)? == false {
                continue
            }

            if config.print_device_properties {
                print_device_properties(&phy_device.property);
            }

            selected_device = Some(phy_device);
            break;
        }

        if let Some(phy_device) = selected_device {

            // get memory properties.
            let memories = unsafe {
                instance.handle.get_physical_device_memory_properties(phy_device.handle)
            };

            let depth_format = query_depth_format(instance, &phy_device);

            let dst_device = VkPhysicalDevice {
                device_name: chars2string(&phy_device.property.device_name),
                handle: phy_device.handle,
                limits: phy_device.property.limits,
                features_enable: enable_feature_if_support(&phy_device, &config),
                config, memories, depth_format,
            };

            Ok(dst_device)
        } else {

            Err(VkError::custom("Failed to find supportive Vulkan device."))
        }
    }

    fn query_phy_devices(instance: &VkInstance, config: &PhysicalDevConfig) -> VkResult<Vec<PhyDeviceTmp>> {

        let alternative_devices = unsafe {
            instance.handle.enumerate_physical_devices()
                .or(Err(VkError::query("Physical Device")))?
        };

        let mut alternative_devices: Vec<PhyDeviceTmp> = alternative_devices.into_iter().map(|phy_device| {
            query_device_property(instance, phy_device)
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

    #[inline]
    pub fn features_enabled(&self) -> &vk::PhysicalDeviceFeatures {
        &self.features_enable
    }

    #[inline]
    pub fn enable_extensions(&self) -> &Vec<CString> {
        &self.config.request_extensions
    }
}

struct PhyDeviceTmp {

    handle: vk::PhysicalDevice,
    property: vk::PhysicalDeviceProperties,
    features: vk::PhysicalDeviceFeatures,
}





// Physical Extension ----------------------------------------------------------------
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DeviceExtensionType {
    Swapchain,
}

impl DeviceExtensionType {

    fn name(&self) -> CString {
        match self {
            | DeviceExtensionType::Swapchain => {
                CString::new("VK_KHR_swapchain").unwrap()
            },
        }
    }
}

fn is_all_extension_support(instance: &VkInstance, phy_device: &PhyDeviceTmp, config: &PhysicalDevConfig) -> VkResult<bool> {

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


// Physical Property -----------------------------------------------------------------
fn query_device_property(instance: &VkInstance, phy_device: vk::PhysicalDevice) -> PhyDeviceTmp {

    let device_property = unsafe {
        instance.handle.get_physical_device_properties(phy_device)
    };

    let available_feature = unsafe {
        instance.handle.get_physical_device_features(phy_device)
    };

    PhyDeviceTmp {
        handle: phy_device,
        property: device_property,
        features: available_feature,
    }
}

fn print_device_properties(property: &vk::PhysicalDeviceProperties) {

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



// Physical Feature ------------------------------------------------------------------
macro_rules! check_feature {
    ($device:ident, $config:ident, $features_enable:ident, {
        $(
           $feature:tt,
        )*
    }) => {

        if $config.print_available_features {
            $(
                println!("{} = {}", stringify!($device.features.$feature), $device.features.$feature);
            )*
        }

        $(
            if $config.request_features.$feature == vk::TRUE {
                if $device.features.$feature == vk::TRUE {
                    $features_enable.$feature = vk::TRUE;
                } else {
                    let device_name = chars2string(&$device.property.device_name);
                    println!("[Warning] Vulkan feature '{}' is not support on {}.", stringify!($feature), device_name);
                }
            }
        )*
    };
}

fn enable_feature_if_support(phy_device: &PhyDeviceTmp, config: &PhysicalDevConfig) -> vk::PhysicalDeviceFeatures {

    let mut features_enable = vk::PhysicalDeviceFeatures::default();

    check_feature!(phy_device, config, features_enable, {
        robust_buffer_access,
        full_draw_index_uint32,
        image_cube_array,
        independent_blend,
        geometry_shader,
        tessellation_shader,
        sample_rate_shading,
        dual_src_blend,
        logic_op,
        multi_draw_indirect,
        draw_indirect_first_instance,
        depth_clamp,
        depth_bias_clamp,
        fill_mode_non_solid,
        depth_bounds,
        wide_lines,
        large_points,
        alpha_to_one,
        multi_viewport,
        sampler_anisotropy,
        texture_compression_etc2,
        texture_compression_astc_ldr,
        texture_compression_bc,
        occlusion_query_precise,
        pipeline_statistics_query,
        vertex_pipeline_stores_and_atomics,
        fragment_stores_and_atomics,
        shader_tessellation_and_geometry_point_size,
        shader_image_gather_extended,
        shader_storage_image_extended_formats,
        shader_storage_image_multisample,
        shader_storage_image_read_without_format,
        shader_storage_image_write_without_format,
        shader_uniform_buffer_array_dynamic_indexing,
        shader_sampled_image_array_dynamic_indexing,
        shader_storage_buffer_array_dynamic_indexing,
        shader_storage_image_array_dynamic_indexing,
        shader_clip_distance,
        shader_cull_distance,
        shader_float64,
        shader_int64,
        shader_int16,
        shader_resource_residency,
        shader_resource_min_lod,
        sparse_binding,
        sparse_residency_buffer,
        sparse_residency_image2_d,
        sparse_residency_image3_d,
        sparse_residency2_samples,
        sparse_residency4_samples,
        sparse_residency8_samples,
        sparse_residency16_samples,
        sparse_residency_aliased,
        variable_multisample_rate,
        inherited_queries,
    });

    features_enable
}
// ----------------------------------------------------------------------------------


// ----------------------------------------------------------------------------------
fn query_depth_format(instance: &VkInstance, phy_device: &PhyDeviceTmp) -> vk::Format {

    // since all depth formats may be optional, we need to find a suitable depth format to use.
    // start with the highest precision packed format.
    let candidates = [
        vk::Format::D32_SFLOAT_S8_UINT,
        vk::Format::D32_SFLOAT,
        vk::Format::D24_UNORM_S8_UINT,
        vk::Format::D16_UNORM_S8_UINT,
        vk::Format::D16_UNORM,
    ];

    for &format in candidates.iter() {
        let format_properties = unsafe {
            instance.handle.get_physical_device_format_properties(phy_device.handle, format)
        };

        // Format must support depth stencil attachment for optimal tiling
        if format_properties.optimal_tiling_features.contains(vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT) {
            return format
        }
    }

    panic!("Failed to find a supported depth format.")
}
// ----------------------------------------------------------------------------------

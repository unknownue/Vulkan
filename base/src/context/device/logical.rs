
use ash::vk;
use ash::version::{DeviceV1_0, InstanceV1_0};

use crate::context::instance::VkInstance;
use crate::context::device::physical::VkPhysicalDevice;
use crate::context::device::queue::{QueueRequester, QueueRequestStrategy};
use crate::error::{VkResult, VkError};
use crate::vkuint;

use std::ptr;


#[derive(Debug, Clone)]
pub struct LogicDevConfig {

    pub request_queues: vk::QueueFlags,
}

impl Default for LogicDevConfig {

    fn default() -> LogicDevConfig {

        LogicDevConfig {
            request_queues: vk::QueueFlags::GRAPHICS | vk::QueueFlags::TRANSFER,
        }
    }
}



pub struct VkLogicalDevice {

    pub handle: ash::Device,
    pub queues: QueryFamilies,
}

pub struct QueryFamilies {
    pub graphics: VkQueue,
    pub compute : VkQueue,
    pub transfer: VkQueue,
}

#[derive(Debug, Clone)]
pub struct VkQueue {
    pub handle: vk::Queue,
    pub family_index: vkuint,
}

impl VkLogicalDevice {

    pub fn new(instance: &VkInstance, phy: &VkPhysicalDevice, config: LogicDevConfig) -> VkResult<VkLogicalDevice> {

        let mut queue_requester = QueueRequester::new(instance, phy, QueueRequestStrategy::ExclusiveQueueCrossFamily);
        let mut queue_requests = QueuesRequestInfo::default();

        if config.request_queues.contains(vk::QueueFlags::GRAPHICS) {
            let graphics_index = queue_requester.request_queue(vk::QueueFlags::GRAPHICS, 1.0)?;
            queue_requests.graphics_index = Some(graphics_index);
        }
        if config.request_queues.contains(vk::QueueFlags::COMPUTE) {
            let compute_index = queue_requester.request_queue(vk::QueueFlags::COMPUTE, 1.0)?;
            queue_requests.compute_index = Some(compute_index);
        }
        if config.request_queues.contains(vk::QueueFlags::TRANSFER) {
            let transfer_index = queue_requester.request_queue(vk::QueueFlags::TRANSFER, 1.0)?;
            queue_requests.transfer_index = Some(transfer_index);
        }

        let queue_cis = queue_requester.queue_cis();

        use crate::utils::cast::cstrings2ptrs;
        let enable_layer_names = cstrings2ptrs(&instance.enable_layer_names);
        let enable_extension_names = cstrings2ptrs(phy.enable_extensions());

        // Create the logical device.
        let device_ci = vk::DeviceCreateInfo {
            s_type                     : vk::StructureType::DEVICE_CREATE_INFO,
            p_next                     : ptr::null(),
            // flags is reserved for future use in API version 1.1.82.
            flags                      : vk::DeviceCreateFlags::empty(),
            queue_create_info_count    : queue_cis.len() as _,
            p_queue_create_infos       : queue_cis.as_ptr(),
            enabled_layer_count        : enable_layer_names.len() as _,
            pp_enabled_layer_names     : enable_layer_names.as_ptr(),
            enabled_extension_count    : enable_extension_names.len() as _,
            pp_enabled_extension_names : enable_extension_names.as_ptr(),
            p_enabled_features         : phy.enable_features(),
        };

        let handle = unsafe {
            instance.handle.create_device(phy.handle, &device_ci, None)
                .or(Err(VkError::create("Logical Device")))?
        };

        let queues = queue_requests.dispatch_queues(&handle, &queue_requester);

        if config.request_queues.contains(vk::QueueFlags::GRAPHICS) {
            debug_assert_ne!(queues.graphics.handle, vk::Queue::null())
        }
        if config.request_queues.contains(vk::QueueFlags::COMPUTE) {
            debug_assert_ne!(queues.compute.handle, vk::Queue::null())
        }
        if config.request_queues.contains(vk::QueueFlags::TRANSFER) {
            debug_assert_ne!(queues.transfer.handle, vk::Queue::null())
        }

        let device = VkLogicalDevice { handle, queues };
        Ok(device)
    }

    pub fn discard(&self) {

        unsafe {
            self.handle.destroy_device(None);
        }
    }
}


#[derive(Default)]
struct QueuesRequestInfo {
    graphics_index: Option<usize>,
    compute_index : Option<usize>,
    transfer_index: Option<usize>,
}

impl QueuesRequestInfo {

    fn dispatch_queues(self, device: &ash::Device, requester: &QueueRequester) -> QueryFamilies {

        let graphics_queue = self.graphics_index.and_then(|graphics_index| {
            Some(requester.dispatch_queue(device, graphics_index))
        }).unwrap_or_default();

        let compute_queue = self.compute_index.and_then(|compute_index| {
            Some(requester.dispatch_queue(device, compute_index))
        }).unwrap_or_default();

        let transfer_queue = self.transfer_index.and_then(|transfer_index| {
            Some(requester.dispatch_queue(device, transfer_index))
        }).unwrap_or_default();

        QueryFamilies {
            graphics: graphics_queue,
            compute : compute_queue,
            transfer: transfer_queue,
        }
    }
}

impl Default for VkQueue {

    fn default() -> VkQueue {
        VkQueue {
            handle: vk::Queue::null(),
            family_index: 0,
        }
    }
}

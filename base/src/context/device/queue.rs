
use ash::vk;
use ash::version::{DeviceV1_0, InstanceV1_0};

use crate::context::instance::VkInstance;
use crate::context::device::physical::VkPhysicalDevice;
use crate::context::device::logical::VkQueue;
use crate::error::{VkResult, VkError};
use crate::{vkfloat, vkuint};

use std::ptr;

type FamilyIndex   = usize;
type QueueIndex    = usize;
type QueuePriority = vkfloat;

pub struct QueueRequester {

    strategy: QueueRequestStrategy,

    // record the current create info of each queue family.
    cis: Vec<FamilyQueuesCreateInfo>,
    // the properties of each queue family queried from Vulkan.
    family_properties: Vec<vk::QueueFamilyProperties>,
    // record the family index and inner queue index of each requested queue.
    queues_requested: Vec<(FamilyIndex, QueueIndex)>,
}

pub enum QueueRequestStrategy {
    ExclusiveQueueCrossFamily,
}

#[derive(Default)]
struct FamilyQueuesCreateInfo {

    count: vkuint,
    priorities: Vec<QueuePriority>,
}

impl QueueRequester {

    pub fn new(instance: &VkInstance, phy: &VkPhysicalDevice, strategy: QueueRequestStrategy) -> QueueRequester {

        let families = unsafe {
            instance.handle.get_physical_device_queue_family_properties(phy.handle)
        };

        let mut queue_cis = Vec::with_capacity(families.len());
        for _ in 0..families.len() {
            queue_cis.push(FamilyQueuesCreateInfo::default());
        }

        QueueRequester {
            strategy,
            cis: queue_cis,
            family_properties: families,
            queues_requested: Vec::new(),
        }
    }

    pub fn request_queue(&mut self, request_queue: vk::QueueFlags, priority: QueuePriority) -> VkResult<usize> {

        // get all support queue families.
        let mut candidate_families = self.candidate_family(request_queue);
        let mut selected_family = None;

        // try to find a dedicated queue family.
        let dedicated_family = candidate_families.iter().enumerate().find_map(|(pos, family_index)| {
            if self.family_properties[family_index.clone()].queue_flags == request_queue {
                Some((family_index.clone() as usize, pos))
            } else {
                None
            }
        });

        // check if there are enough queues remain in dedicated queue family.
        if let Some((dedicated_family_index, position)) = dedicated_family {

            if self.cis[dedicated_family_index].count < self.family_properties[dedicated_family_index].queue_count {
                // set dedicated_family_index as the final selected queue family.
                selected_family = Some(dedicated_family_index);
            } else {
                // if there are no queue remaining in this family.
                // remove this candidate queue family.
                candidate_families.remove(position);
            }
        }

        if selected_family.is_none() {

            // select the first family which has remaining queue.
            selected_family = candidate_families.iter().find(|&&family_index| {
                self.cis[family_index].count < self.family_properties[family_index].queue_count
            }).cloned();
        }

        if let Some(final_family) = selected_family {

            let queue_index = self.cis[final_family].count;

            // update queue family counts.
            self.cis[final_family].count += 1;
            self.cis[final_family].priorities.push(priority);

            let requested_index = self.queues_requested.len();
            self.queues_requested.push((final_family, queue_index as usize));

            Ok(requested_index)
        } else {
            Err(VkError::other(format!("Request Queue with flags({:?}) is not support on current Vulkan device.", request_queue)))
        }
    }

    fn candidate_family(&self, request_queue: vk::QueueFlags) -> Vec<FamilyIndex> {

        self.family_properties.iter().enumerate().filter_map(|(i, family)| {

            if family.queue_flags.contains(request_queue) {
                Some(i)
            } else {
                None
            }
        }).collect()
    }

    pub fn queue_cis(&self) -> Vec<vk::DeviceQueueCreateInfo> {

        self.cis.iter().enumerate().filter_map(|(family_index, ci)| {

            if ci.count > 0 {
                let device_queue_ci = vk::DeviceQueueCreateInfo {
                    s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                    p_next: ptr::null(),
                    // flags is reserved for future use in API version 1.1.82.
                    flags : vk::DeviceQueueCreateFlags::empty(),
                    queue_family_index: family_index as _,
                    queue_count       : ci.priorities.len() as _,
                    p_queue_priorities: ci.priorities.as_ptr(),
                };
                Some(device_queue_ci)
            } else {
                None
            }
        }).collect()
    }

    pub fn dispatch_queue(&self, device: &ash::Device, queue_request_index: usize) -> VkQueue {

        let (family_index, queue_index) = self.queues_requested[queue_request_index];

        let handle = unsafe {
            device.get_device_queue(family_index as vkuint, queue_index as vkuint)
        };

        VkQueue {
            handle,
            family_index: family_index as _,
        }
    }
}

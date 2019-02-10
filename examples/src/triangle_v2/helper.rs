
use ash::vk;
use ash::version::DeviceV1_0;

use vkbase::context::VkDevice;
use vkbase::{VkResult, VkError};
use vkbase::utils::time::VkTimeDuration;
use vkbase::vkuint;

use std::ptr;


/// This function is used to request a device memory type that supports all the property flags we request (e.g. device local, host visible).
///
/// Upon success it will return the index of the memory type that fits our request memory properties.
///
/// This is necessary as implementations can offer an arbitrary number of memory types with different memory properties.
///
/// You can check http://vulkan.gpuinfo.org/ for details on different memory configurations.
pub fn get_memory_type_index(device: &VkDevice, mut type_bits: vkuint, properties: vk::MemoryPropertyFlags) -> vkuint {

    // Iterate over all memory types available for the device used in this example.
    let memories = &device.phy.memories;
    for i in 0..memories.memory_type_count {
        if (type_bits & 1) == 1 {
            if memories.memory_types[i as usize].property_flags.contains(properties) {
                return i
            }
        }

        type_bits >>= 1;
    }

    panic!("Could not find a suitable memory type")
}

pub fn create_command_pool(device: &VkDevice) -> VkResult<vk::CommandPool> {

    let command_pool_ci = vk::CommandPoolCreateInfo {
        s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
        queue_family_index: device.logic.queues.graphics.family_index,
    };

    let pool = unsafe {
        device.logic.handle.create_command_pool(&command_pool_ci, None)
            .map_err(|_| VkError::create("Command Pool"))?
    };
    Ok(pool)
}

/// Get a new command buffer from the command pool.
///
/// If begin is true, the command buffer is also started so we can start adding commands.
pub fn create_command_buffer(device: &VkDevice, pool: vk::CommandPool, is_begin: bool) -> VkResult<vk::CommandBuffer> {

    let command_buffer_ci = vk::CommandBufferAllocateInfo {
        s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
        p_next: ptr::null(),
        command_pool: pool,
        level: vk::CommandBufferLevel::PRIMARY,
        command_buffer_count: 1,
    };

    let mut buffers = unsafe {
        device.logic.handle.allocate_command_buffers(&command_buffer_ci)
            .map_err(|_| VkError::create("Command Buffers"))?
    };
    let cmd_buffer = buffers.pop().unwrap();

    // If requested, also start the new command buffer
    if is_begin {
        let begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            flags: vk::CommandBufferUsageFlags::empty(),
            p_inheritance_info: ptr::null(),
        };

        unsafe {
            device.logic.handle.begin_command_buffer(cmd_buffer, &begin_info)
                .map_err(|_| VkError::device("Begin Command Buffer"))?
        }
    }

    Ok(cmd_buffer)
}

pub fn flush_command_buffer(device: &VkDevice, pool: vk::CommandPool, command: vk::CommandBuffer) -> VkResult<()> {

    debug_assert_ne!(command, vk::CommandBuffer::null());

    unsafe {
        device.logic.handle.end_command_buffer(command)
            .map_err(|_| VkError::create("End Command Buffer"))?;
    }

    let submit_info = vk::SubmitInfo {
        s_type: vk::StructureType::SUBMIT_INFO,
        p_next: ptr::null(),
        wait_semaphore_count   : 0,
        p_wait_semaphores      : ptr::null(),
        p_wait_dst_stage_mask  : ptr::null(),
        command_buffer_count   : 1,
        p_command_buffers      : &command,
        signal_semaphore_count : 0,
        p_signal_semaphores    : ptr::null(),
    };

    // Create fence to ensure that the command buffer has finished executing.
    let fence_ci = vk::FenceCreateInfo {
        s_type: vk::StructureType::FENCE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::FenceCreateFlags::empty(),
    };

    unsafe {

        let fence = device.logic.handle.create_fence(&fence_ci, None)
            .map_err(|_| VkError::create("Fence"))?;

        // Submit to the queue.
        device.logic.handle.queue_submit(device.logic.queues.graphics.handle, &[submit_info], fence)
            .map_err(|_| VkError::device("Queue Submit"))?;

        // Wait for the fence to signal that command buffer has finished executing.
        device.logic.handle.wait_for_fences(&[fence], true, VkTimeDuration::Infinite.into())
            .map_err(|_| VkError::device("Wait for fences"))?;

        device.logic.handle.destroy_fence(fence, None);
        device.logic.handle.free_command_buffers(pool, &[command]);
    }

    Ok(())
}

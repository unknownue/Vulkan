
use ash::vk;
use ash::version::DeviceV1_0;

use vkbase::context::VkDevice;
use vkbase::ci::VkObjectBuildableCI;
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

    use vkbase::ci::command::CommandPoolCI;

    let command_pool = CommandPoolCI::new(device.logic.queues.graphics.family_index)
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .build(device)?;
    Ok(command_pool)
}

/// Get a new command buffer from the command pool.
///
/// If begin is true, the command buffer is also started so we can start adding commands.
pub fn create_command_buffer(device: &VkDevice, pool: vk::CommandPool, is_begin: bool) -> VkResult<vk::CommandBuffer> {

    use vkbase::ci::command::CommandBufferAI;

    let cmd_buffer = CommandBufferAI::new(pool, 1)
        .build(device)?
        .remove(0);

    // If requested, also start the new command buffer.
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

    use vkbase::ci::sync::FenceCI;

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
    let fence = FenceCI::new(false).build(device)?;

    unsafe {
        // Submit to the queue.
        device.logic.handle.queue_submit(device.logic.queues.graphics.handle, &[submit_info], fence)
            .map_err(|_| VkError::device("Queue Submit"))?;

        // Wait for the fence to signal that command buffer has finished executing.
        device.logic.handle.wait_for_fences(&[fence], true, VkTimeDuration::Infinite.into())
            .map_err(|_| VkError::device("Wait for fences"))?;

        device.logic.handle.free_command_buffers(pool, &[command]);
        device.discard(fence);
    }
    Ok(())
}


use ash::vk;

use crate::context::VkDevice;
use crate::{vkuint, vkbytes};

pub struct MemorySlice<T> where T: Copy {

    /// the handle of vk::Buffer or vk::Image.
    pub handle: T,
    /// the starting offset of this memory slice.
    pub offset: vkbytes,
    /// the size of this memory slice.
    pub size  : vkbytes,
}

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

#[inline]
pub fn bound_to_alignment(bound_value: vkbytes, alignment: vkbytes) -> vkbytes {

    // `!` operator will make 1 to 0 or make 0 to 1 for each bit for any integer type.
    (bound_value + alignment - 1) & !(alignment - 1)
}

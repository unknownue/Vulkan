
use ash::vk;

use crate::context::VkDevice;
use crate::{vkuint, vkbytes};
use std::ops::{Add, Sub, Not, BitAnd};


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


pub trait IntegerAlignable: Copy + Add<Output=Self> + Sub<Output=Self> + Not<Output=Self> + BitAnd<Output=Self> {
    const INTEGER_UNIT: Self;

    /// align an integer to a specific alignment.
    ///
    /// `alignment` must be the power of 2.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::utils::IntegerAlignable;
    ///
    /// assert_eq!(127.align_to(128), 128);
    /// assert_eq!(129.align_to(128), 256);
    /// ```
    fn align_to(self, alignment: Self) -> Self {

        // `!` operator will make 1 to 0 or make 0 to 1 for each bit for any integer type.
        (self + alignment - Self::INTEGER_UNIT) & !(alignment - Self::INTEGER_UNIT)
    }
}

macro_rules! align_impl {
    ($($t:ty)*) => ($(

        impl IntegerAlignable for $t {
            const INTEGER_UNIT: $t = 1;
        }
    )*)
}

align_impl! { usize u8 u16 u32 u64 u128 isize i8 i16 i32 i64 i128 }

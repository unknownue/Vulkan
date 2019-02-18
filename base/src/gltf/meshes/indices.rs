
use crate::gltf::asset::GltfDocument;
use crate::ci::buffer::BufferCI;
use crate::error::{VkResult, VkError};

use crate::{vkuint, vkbytes, vkptr};

pub struct IndicesData {

    start_index: u32,
    data_content: Vec<vkuint>,
}

pub struct IndicesExtendInfo {

    pub first_index  : vkuint,
    pub indices_count: vkuint,
}

impl IndicesData {

    pub fn extend(&mut self, primitive: &gltf::Primitive, source: &GltfDocument) -> VkResult<IndicesExtendInfo> {

        let reader = primitive.reader(|b| Some(&source.buffers[b.index()]));
        let indices_range = get_indices_range(primitive)?;

        let start_index = self.start_index.clone();

        // TODO: Support other integer type.
        let index_iter = reader.read_indices()
            .ok_or(VkError::custom("Missing indices property in glTF primitive."))?
            .into_u32()
            .map(move |index_element| index_element + start_index);

        let result = IndicesExtendInfo {
            first_index  : self.start_index,
            indices_count: index_iter.size_hint().0 as _,
        };

        self.data_content.extend(index_iter);
        self.start_index += indices_range as u32;

        Ok(result)
    }

    pub fn buffer_ci(&self) -> Option<BufferCI> {

        if self.start_index > 0 {

            let indices_size = (self.data_content.len() * ::std::mem::size_of::<vkuint>()) as vkbytes;
            Some(BufferCI::new(indices_size))
        } else {
            None
        }
    }

    pub fn map_data(&self, memory_ptr: vkptr) {

        unsafe {

            let mapped_copy_target = ::std::slice::from_raw_parts_mut(memory_ptr as *mut vkuint, self.data_content.len());
            mapped_copy_target.copy_from_slice(&self.data_content);
        }
    }
}

impl Default for IndicesData {

    fn default() -> IndicesData {
        IndicesData {
            start_index: 0,
            data_content: Vec::new(),
        }
    }
}

fn get_indices_range(primitive: &gltf::Primitive) -> VkResult<u64> {

    let indices_accessor = primitive.indices()
        .ok_or(VkError::custom("Failed to get indices property of glTF::Primitive"))?;

    // Get the maximum index of this primitive.
    let index_max = get_index(indices_accessor.max())?;
    // Get the minimum index of this primitive.
    let index_min = get_index(indices_accessor.min())?;
    // calculate the range of these indices.
    let indices_range = index_max - index_min + 1;

    Ok(indices_range)
}

fn get_index(value: Option<gltf::json::Value>) -> VkResult<u64> {

    value
        .and_then(|v| v.as_array().cloned())
        .and_then(|v| v.first().cloned())
        .and_then(|v| v.as_u64())
        .ok_or(VkError::custom("Invalid or missing min/max property for indices property."))
}

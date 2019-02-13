
use serde_derive::Serialize;

use crate::error::{VkResult, VkError};
use crate::vkfloat;

pub type MatSerializedData = Vec<u8>;

// ------------------------------------------------------------------------------------
pub struct MaterialData {

    pbr: PbrMetallicRoughness,
    emissive_factor: [vkfloat; 3],
}

struct PbrMetallicRoughness {

    base_color_factor: [vkfloat; 4],
    metallic_factor  : vkfloat,
}

impl From<&'_ gltf::Material<'_>> for MaterialData {

    fn from(raw_material: &gltf::Material) -> MaterialData {

        let raw_pbr = raw_material.pbr_metallic_roughness();

        MaterialData {
            pbr: PbrMetallicRoughness {
                base_color_factor: raw_pbr.base_color_factor(),
                metallic_factor  : raw_pbr.metallic_factor(),
            },
            emissive_factor: raw_material.emissive_factor(),
        }
    }
}

impl MaterialData {

    pub fn serialize(&self) -> VkResult<MatSerializedData> {

        let data = MaterialConstants {
            base_color_factor: self.pbr.base_color_factor,
            metallic_factor  : self.pbr.metallic_factor,
            emissive_factor  : self.emissive_factor,
        };

        let bytes_data = bincode::serialize(&data)
            .map_err(VkError::serialize)?;

        Ok(bytes_data)
    }
}
// ------------------------------------------------------------------------------------

// ------------------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, Serialize)]
pub struct MaterialConstants {
    base_color_factor: [vkfloat; 4],
    emissive_factor  : [vkfloat; 3],
    metallic_factor  : vkfloat,
}
// ------------------------------------------------------------------------------------

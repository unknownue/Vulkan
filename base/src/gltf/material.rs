
use serde_derive::Serialize;
use std::collections::HashMap;

use crate::gltf::asset::{GltfDocument, AssetAbstract};
use crate::gltf::asset::ReferenceIndex;
use crate::gltf::scene::Scene;
use crate::error::{VkResult, VkError};
use crate::{vkfloat, vkuint};

pub type MatSerializedData = Vec<u8>;
pub type MaterialSlice<'a> = &'a [u8];
pub type MaterialResource = MaterialAsset;

const DEFAULT_MATERIAL_INDEX : usize = usize::max_value();
const DEFAULT_MATERIAL_OFFSET: usize = 0;
const MATERIAL_SIZE: usize = ::std::mem::size_of::<MaterialData>();
type MaterialOffset = usize;

// ------------------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, Serialize)]
struct MaterialData {

    base_color_factor: [vkfloat; 4],
    emissive_factor  : [vkfloat; 3],
    metallic_factor  : vkfloat,
}

impl Default for MaterialData {

    fn default() -> MaterialData {
        MaterialData {
            base_color_factor: [1.0; 4],
            emissive_factor: [0.0; 3],
            metallic_factor: 1.0,
        }
    }
}

impl From<gltf::Material<'_>> for MaterialData {

    fn from(raw_material: gltf::Material) -> MaterialData {

        let raw_pbr = raw_material.pbr_metallic_roughness();

        MaterialData {
            base_color_factor : raw_pbr.base_color_factor(),
            metallic_factor   : raw_pbr.metallic_factor(),
            emissive_factor   : raw_material.emissive_factor(),
        }
    }
}

impl MaterialData {

    pub fn serialize(&self) -> VkResult<MatSerializedData> {

        let bytes_data = bincode::serialize(self)
            .map_err(VkError::serialize)?;

        Ok(bytes_data)
    }
}
// ------------------------------------------------------------------------------------


// ------------------------------------------------------------------------------------
pub struct MaterialAsset {

    data_content: MatSerializedData,
    material_count: usize,

    material_mapping: HashMap<ReferenceIndex, MaterialOffset>,
}

impl MaterialAsset {

    pub fn new() -> VkResult<MaterialAsset> {

        let default_material = MaterialData::default();
        let data_content = default_material.serialize()?;

        let mut material_mapping = HashMap::new();
        // the offset of default material data is 0.
        material_mapping.insert(DEFAULT_MATERIAL_INDEX, DEFAULT_MATERIAL_OFFSET);
        let material_count = 1;

        let result = MaterialAsset { data_content, material_count, material_mapping };
        Ok(result)
    }

    pub fn material_size(&self) -> vkuint {
        MATERIAL_SIZE as vkuint
    }

    pub fn get_material_serialized(&self, material_index: &Option<ReferenceIndex>) -> MaterialSlice {

        let offset = self.material_mapping.get(&material_index.unwrap_or(DEFAULT_MATERIAL_INDEX)).cloned()
            .unwrap_or(DEFAULT_MATERIAL_OFFSET);
        &self.data_content[offset..(offset + MATERIAL_SIZE)]
    }
}

impl AssetAbstract for MaterialAsset {
    const ASSET_NAME: &'static str = "Materials";

    fn read_doc(&mut self, source: &GltfDocument, _scene: &Scene) -> VkResult<()> {

        for doc_material in source.doc.materials() {

            if let Some(json_index) = doc_material.index() {

                let material = MaterialData::from(doc_material);
                let material_serialized = material.serialize()?;
                self.data_content.extend(material_serialized);

                let material_offset = self.material_count * MATERIAL_SIZE;
                self.material_mapping.insert(json_index, material_offset);

                self.material_count += 1;
            }
        }

        Ok(())
    }
}
// ------------------------------------------------------------------------------------


use serde_derive::Serialize;

use crate::gltf::asset::{GltfDocument, AssetAbstract, AssetElementList};
use crate::error::{VkResult, VkError};
use crate::vkfloat;

pub type MatSerializedData = Vec<u8>;
const DEFAULT_MATERIAL_INDEX: usize = usize::max_value();

// ------------------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, Serialize)]
pub struct MaterialData {

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

    materials: AssetElementList<MatSerializedData>,
}

impl MaterialAsset {

    pub fn new() -> VkResult<MaterialAsset> {

        let mut materials = AssetElementList::default();
        let default_material = MaterialData::default().serialize()?;
        materials.push(DEFAULT_MATERIAL_INDEX, default_material);

        let result = MaterialAsset { materials };
        Ok(result)
    }
}

impl<'a> AssetAbstract<'a> for MaterialAsset {
    const ASSET_NAME: &'static str = "Materials";

    fn read_doc(&mut self, source: &GltfDocument) -> VkResult<()> {

        for doc_material in source.doc.materials() {

            if let Some(json_index) = doc_material.index() {

                let material = MaterialData::from(doc_material);
                let material_serialized = material.serialize()?;

                self.materials.push(json_index, material_serialized);
            }
        }

        Ok(())
    }
}
// ------------------------------------------------------------------------------------

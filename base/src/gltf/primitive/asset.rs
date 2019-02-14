
use crate::gltf::asset::{GltfDocument, AssetAbstract};
use crate::gltf::asset::{ReferenceIndex, StorageIndex};
use crate::gltf::mesh::Mesh;
use crate::gltf::primitive::RenderParams;
use crate::gltf::primitive::attributes::{AttributesData, AttributeFlags};
use crate::gltf::primitive::indices::IndicesData;
use crate::error::{VkResult, VkTryFrom};

use std::collections::HashMap;

pub struct MeshAsset {

    attributes: AttributesData,
    indices: IndicesData,
    meshes: Vec<Mesh>,

    query_table: HashMap<ReferenceIndex, StorageIndex>,
}

impl VkTryFrom<AttributeFlags> for MeshAsset {

    fn try_from(flag: AttributeFlags) -> VkResult<MeshAsset> {

        let result = MeshAsset {
            attributes: AttributesData::try_from(flag)?,
            indices: IndicesData::default(),
            meshes: Vec::new(),
            query_table: HashMap::new(),
        };
        Ok(result)
    }
}

impl<'a> AssetAbstract<'a> for MeshAsset {
    const ASSET_NAME: &'static str = "Meshes";

    type AssetElement = Mesh;

    fn read_doc(&mut self, source: &GltfDocument) -> VkResult<()> {

        for doc_mesh in source.doc.meshes() {

            let json_index = doc_mesh.index();
            let storage_index = self.meshes.len();
            self.query_table.insert(json_index, storage_index);

            let mesh = Mesh::from_doc(doc_mesh, source, &mut self.attributes, &mut self.indices)?;
            self.meshes.push(mesh);
        }

        Ok(())
    }

    fn asset_at(&mut self, ref_index: ReferenceIndex) -> Option<&Self::AssetElement> {

        if let Some(storage_index) = self.query_table.get(&ref_index).cloned() {
            Some(&self.meshes[storage_index])
        } else {
            None
        }
    }
}

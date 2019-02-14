
use crate::gltf::meshes::{MeshAsset, AttributeFlags};
use crate::gltf::nodes::{NodeAsset, NodeAttachmentFlags};
use crate::error::{VkResult, VkTryFrom};

use std::collections::HashMap;

pub type ReferenceIndex = usize;
pub type   StorageIndex = usize;

// --------------------------------------------------------------------------------------
pub struct GltfDocument {
    pub doc: gltf::Document,
    pub buffers: Vec<gltf::buffer::Data>,
    pub images : Vec<gltf::image::Data>,
}
// --------------------------------------------------------------------------------------

// --------------------------------------------------------------------------------------
pub trait AssetAbstract<'a>: Sized {
    const ASSET_NAME: &'static str;

    fn read_doc(&mut self, source: &GltfDocument) -> VkResult<()>;
}

pub struct AssetElementList<T> {

    list: Vec<T>,
    query_table: HashMap<ReferenceIndex, StorageIndex>,
}

impl<T> Default for AssetElementList<T> {

    fn default() -> AssetElementList<T> {
        AssetElementList {
            list: Vec::new(),
            query_table: HashMap::new(),
        }
    }
}

impl<T> AssetElementList<T> {

    pub fn push(&mut self, ref_index: ReferenceIndex, element: T) {

        let storage_index = self.list.len();
        self.query_table.insert(ref_index, storage_index);

        self.list.push(element);
    }

    pub fn asset_at(&self, ref_index: ReferenceIndex) -> &T {

        debug_assert!(self.query_table.contains_key(&ref_index));

        let storage_index = self.query_table.get(&ref_index).cloned().unwrap();
        &self.list[storage_index]
    }
}
// --------------------------------------------------------------------------------------

// --------------------------------------------------------------------------------------
pub struct AssetRepository {
    pub nodes : NodeAsset,
    pub meshes: MeshAsset,
}

impl AssetRepository {

    pub fn new(attr_flag: AttributeFlags, attachment_flag: NodeAttachmentFlags) -> VkResult<AssetRepository> {

        let repository = AssetRepository {
            nodes : NodeAsset::try_from(attachment_flag)?,
            meshes: MeshAsset::try_from(attr_flag)?,
        };
        Ok(repository)
    }
}
// --------------------------------------------------------------------------------------


use crate::gltf::meshes::{MeshAsset, AttributeFlags};
use crate::gltf::nodes::{NodeAsset, NodeAttachmentFlags};
use crate::error::{VkResult, VkTryFrom};

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

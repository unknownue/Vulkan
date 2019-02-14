
use crate::gltf::meshes::{MeshAsset, AttributeFlags};
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
    type AssetElement;

    fn read_doc(&mut self, source: &GltfDocument) -> VkResult<()>;

    fn asset_at(&mut self, ref_index: ReferenceIndex) -> Option<&Self::AssetElement>;
}
// --------------------------------------------------------------------------------------

// --------------------------------------------------------------------------------------
pub struct AssetRepository {
    pub meshes: MeshAsset,
}

impl AssetRepository {

    pub fn new(attr_flag: AttributeFlags) -> VkResult<AssetRepository> {

        let repository = AssetRepository {
            meshes: MeshAsset::try_from(attr_flag)?,
        };
        Ok(repository)
    }
}
// --------------------------------------------------------------------------------------


use crate::error::{VkResult, VkTryFrom};

use std::collections::HashMap;

type ReferenceIndex = usize;
type   StorageIndex = usize;

pub struct GltfDocument {
    pub doc: gltf::Document,
    pub buffers: Vec<gltf::buffer::Data>,
    pub images : Vec<gltf::image::Data>,
}


pub struct AssetRepo<Asset> {

    indices: HashMap<ReferenceIndex, StorageIndex>,
    asset: Asset,
}

impl<Asset, Any> VkTryFrom<Any> for AssetRepo<Asset>
    where
        Asset: VkTryFrom<Any> {

    fn try_from(any: Any) -> VkResult<AssetRepo<Asset>> {

        let result = AssetRepo {
            indices: HashMap::new(),
            asset  : Asset::try_from(any)?,
        };
        Ok(result)
    }
}

impl<'a, Asset> AssetRepo<Asset>
    where
        Asset: AssetAbstract<'a> {

    pub fn read_doc(&mut self, doc: Asset::DocumentType, source: &GltfDocument, ref_index: ReferenceIndex) -> VkResult<Asset::AssetInfo> {

        let store_index = if let Some(store_index) = self.indices.get(&ref_index) {

            store_index.clone()
        } else {
            let store_index = self.asset.extend(doc, source)?;

            self.indices.insert(ref_index, store_index);
            store_index
        };

        let result = self.asset.asset_info(store_index);
        Ok(result)
    }
}


pub trait AssetAbstract<'a>: Sized {
    const ASSET_NAME: &'static str;
    type DocumentType;
    type AssetInfo;

    fn extend(&mut self, doc: Self::DocumentType, source: &GltfDocument) -> VkResult<StorageIndex>;

    fn asset_info(&self, at: StorageIndex) -> Self::AssetInfo;
}

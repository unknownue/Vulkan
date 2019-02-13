
use crate::gltf::asset::{GltfDocument, AssetAbstract, StorageIndex};
use crate::gltf::primitive::RenderParams;
use crate::gltf::primitive::attributes::{AttributesData, AttributeFlags};
use crate::gltf::primitive::indices::IndicesData;
use crate::error::{VkResult, VkTryFrom};

use std::collections::HashMap;

type PrimitiveCount = usize;
type PrimitiveStart = usize;

pub struct MeshAsset {

    attributes: AttributesData,
    indices: IndicesData,
    params: Vec<RenderParams>,

    primitive_ranges: HashMap<StorageIndex, (PrimitiveStart, PrimitiveCount)>,
}

impl VkTryFrom<AttributeFlags> for MeshAsset {

    fn try_from(flag: AttributeFlags) -> VkResult<MeshAsset> {

        let result = MeshAsset {
            attributes: AttributesData::try_from(flag)?,
            indices: IndicesData::default(),
            params: Vec::new(),
            primitive_ranges: HashMap::new(),
        };
        Ok(result)
    }
}

impl<'a> AssetAbstract<'a> for MeshAsset {
    const ASSET_NAME: &'static str = "Meshes";

    type DocumentType = &'a gltf::Mesh<'a>;
    type AssetInfo    = Vec<RenderParams>;

    fn extend(&mut self, doc: Self::DocumentType, source: &GltfDocument) -> VkResult<StorageIndex> {

        let primitive_iter = doc.primitives();

        let primitive_start = self.params.len();
        let primitive_count = primitive_iter.size_hint().0;

        self.params.reserve(primitive_count);

        for primitive in primitive_iter {

            // read vertex attribute data of glTF::Primitive.
            let attribute_info = self.attributes.data_content.extend(&primitive, source);

            let render_params = match primitive.indices() {
                | None => {
                    // set the draw method of this primitive to drawArray.
                    RenderParams::DrawArray {
                        first_vertex: attribute_info.first_vertex as _,
                        vertex_count: attribute_info.vertex_count as _,
                    }
                },
                | Some(_) => {
                    // read indices data of glTF::Primitive.
                    let indices_info = self.indices.extend(&primitive, source)?;
                    // set the draw method of this primitive to drawIndexed.
                    RenderParams::DrawIndex {
                        first_index: indices_info.first_index,
                        index_count: indices_info.indices_count,
                    }
                },
            };

            self.params.push(render_params);
        }

        let store_index = self.primitive_ranges.len();
        self.primitive_ranges.insert(store_index, (primitive_start, primitive_count));

        Ok(store_index)
    }

    fn asset_info(&self, at: StorageIndex) -> Self::AssetInfo {

        // unwrap() is ok here.
        let (primitive_start, primitive_count) = self.primitive_ranges.get(&at).unwrap().clone();

        self.params
            .get(primitive_start..(primitive_start + primitive_count))
            .to_owned().unwrap()
            .to_vec()
    }
}

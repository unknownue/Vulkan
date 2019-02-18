
use crate::gltf::scene::Scene;
use crate::gltf::nodes::NodeAttachmentFlags;
use crate::gltf::meshes::AttributeFlags;
use crate::gltf::asset::{GltfDocument, AssetAbstract, AssetRepository};
use crate::error::{VkResult, VkError, VkErrorKind};

use std::path::Path;

pub struct GltfModelInfo<'a> {
    pub path: &'a Path,
    pub attribute: AttributeFlags,
    pub node: NodeAttachmentFlags,
}

fn load_gltf(info: GltfModelInfo) -> VkResult<()> {

    let (doc, buffers, images) = gltf::import(info.path)
        .map_err(VkErrorKind::ParseGltf)?;
    let document = GltfDocument { doc, buffers, images };

    // Only support loading the default scene or first scene in glTF file.
    let dst_scene = document.doc.default_scene()
        .or(document.doc.scenes().next())
        .ok_or(VkError::custom("glTF Scene is missing."))?;

    let scene = Scene::from_doc(dst_scene);
    let mut asset_repo = AssetRepository::new(info.attribute, info.node)?;
    asset_repo.meshes.read_doc(&document, &scene)?;
    asset_repo.nodes.read_doc(&document, &scene)?;
    asset_repo.materials.read_doc(&document, &scene)?;

    unimplemented!()
}


use std::path::Path;

use crate::gltf::scene::Scene;
use crate::gltf::nodes::NodeAttachmentFlags;
use crate::gltf::meshes::AttributeFlags;
use crate::gltf::asset::{GltfDocument, AssetAbstract, AssetRepository};
use crate::gltf::asset::VkglTFModel;

use crate::context::VkDevice;
use crate::error::{VkResult, VkError, VkErrorKind};
use crate::Mat4F;


pub struct GltfModelInfo<'a> {
    /// The path of model file.
    pub path: &'a Path,
    /// Indicate what attributes will be read from this model file.
    pub attribute: AttributeFlags,
    /// Indicate what properties will be read for Node hierarchy(etc. transform matrix).
    pub node: NodeAttachmentFlags,
    /// A matrix that will apply to position attribute of the model.
    pub transform: Option<Mat4F>,
}

pub fn load_gltf(device: &mut VkDevice, info: GltfModelInfo) -> VkResult<VkglTFModel> {

    let (doc, buffers, images) = gltf::import(info.path)
        .map_err(VkErrorKind::ParseGltf)?;
    let document = GltfDocument {
        doc, buffers, images,
        transform: info.transform,
    };

    // Only support loading the default scene or first scene in glTF file.
    let dst_scene = document.doc.default_scene()
        .or(document.doc.scenes().next())
        .ok_or(VkError::custom("glTF Scene is missing."))?;

    let scene = Scene::from_doc(dst_scene);
    let mut asset_repo = AssetRepository::new(info.attribute, info.node)?;
    asset_repo.meshes.read_doc(&document, &scene)?;
    asset_repo.nodes.read_doc(&document, &scene)?;
    asset_repo.materials.read_doc(&document, &scene)?;

    let result = asset_repo.allocate(device, scene)?;
    Ok(result)
}


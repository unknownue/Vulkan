
use crate::gltf::asset::GltfDocument;
use crate::gltf::meshes::primitive::Primitive;
use crate::gltf::meshes::attributes::AttributesData;
use crate::gltf::meshes::indices::IndicesData;
use crate::error::VkResult;

// --------------------------------------------------------------------------------------
/// A wrapper class for mesh level in glTF, containing the render parameters read from glTF file.
#[derive(Debug, Clone)]
pub struct Mesh {

    /// a mesh may contain multiple glTF::Primitive.
    primitives: Vec<Primitive>,
}

impl Mesh {

    pub fn from_doc(doc_mesh: gltf::Mesh, source: &GltfDocument, attributes: &mut AttributesData, indices: &mut IndicesData) -> VkResult<Mesh> {

        let mesh_iter = doc_mesh.primitives();
        let mut primitives = Vec::with_capacity(mesh_iter.size_hint().0);

        for doc_primitive in mesh_iter {

            let primitive = Primitive::from_doc(doc_primitive, source, attributes, indices)?;
            primitives.push(primitive);
        }

        let mesh = Mesh { primitives };
        Ok(mesh)
    }
}
// --------------------------------------------------------------------------------------

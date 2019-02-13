
mod attributes;
mod indices;
mod asset;


use crate::gltf::material::{MatSerializedData, MaterialData};
use crate::{VkResult, VkError};
use crate::vkuint;

// --------------------------------------------------------------------------------------
/// A wrapper class for primitive level in glTF, containing the render parameters read from glTF file.
#[derive(Debug, Clone)]
pub struct Primitive {

    /// the draw parameters used in rendering.
    params: RenderParams,
    /// the material data serialized into bytes.
    material: MatSerializedData,
}

impl Primitive {

    pub fn from_doc(doc_primitive: gltf::Primitive) -> VkResult<Primitive> {

        if doc_primitive.mode() != gltf::mesh::Mode::Triangles {
            // Currently only support triangle topology.
            return Err(VkError::unimplemented(format!("{} render mode.", translate_draw_mode(doc_primitive.mode()))))
        }

        // load material data.
        let doc_material = MaterialData::from(&doc_primitive.material());
        let material_serialized = doc_material.serialize()?;

        let result = Primitive {
            params  : RenderParams::Unset,
            material: material_serialized,
        };
        Ok(result)
    }

    pub fn set_render_params(&mut self, value: RenderParams) {
        self.params = value;
    }
}
// --------------------------------------------------------------------------------------

// --------------------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub enum RenderParams {
    DrawArray { vertex_count: vkuint, first_vertex: vkuint },
    DrawIndex {  index_count: vkuint,  first_index: vkuint },
    Unset,
}
// --------------------------------------------------------------------------------------



// --------------------------------------------------------------------------------------
fn translate_draw_mode(from: gltf::mesh::Mode) -> &'static str {

    use gltf::mesh::Mode::*;

    match from {
        | Points        => "Points",
        | Lines         => "Lines",
        | LineLoop      => "LineLoop",
        | LineStrip     => "LineStrip",
        | Triangles     => "Triangles",
        | TriangleStrip => "TriangleStrip",
        | TriangleFan   => "TriangleFan",
    }
}
// --------------------------------------------------------------------------------------

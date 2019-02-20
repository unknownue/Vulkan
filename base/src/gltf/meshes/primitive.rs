
use crate::gltf::asset::{GltfDocument, ReferenceIndex};
use crate::gltf::asset::{VkglTFModel, ModelRenderParams};

use crate::gltf::meshes::attributes::AttributesData;
use crate::gltf::meshes::indices::IndicesData;

use crate::gltf::material::DEFAULT_MATERIAL_INDEX;
use crate::command::{VkCmdRecorder, IGraphics, CmdGraphicsApi};

use crate::{VkResult, VkError};
use crate::vkuint;

// --------------------------------------------------------------------------------------
/// A wrapper class for primitive level in glTF, containing the render parameters read from glTF file.
#[derive(Debug, Clone)]
pub struct Primitive {

    /// the draw parameters used in rendering.
    params: RenderParams,
    /// the json index of material of this primitive.
    material: Option<ReferenceIndex>,
}

impl Primitive {

    pub fn from_doc(doc_primitive: gltf::Primitive, source: &GltfDocument, attributes: &mut AttributesData, indices: &mut IndicesData) -> VkResult<Primitive> {

        if doc_primitive.mode() != gltf::mesh::Mode::Triangles {
            // Currently only support triangle topology.
            return Err(VkError::unimplemented(format!("{} render mode.", translate_draw_mode(doc_primitive.mode()))))
        }

        // read vertex attribute data of glTF::Primitive.
        let attribute_info = attributes.data_content.extend(&doc_primitive, source);

        let render_params = match doc_primitive.indices() {
            | None => {
                // set the draw method of this primitive to drawArray.
                RenderParams::DrawArray {
                    first_vertex: attribute_info.first_vertex as _,
                    vertex_count: attribute_info.vertex_count as _,
                }
            },
            | Some(_) => {
                // read indices data of glTF::Primitive.
                let indices_info = indices.extend(&doc_primitive, source)?;
                // set the draw method of this primitive to drawIndexed.
                RenderParams::DrawIndex {
                    first_index: indices_info.first_index,
                    index_count: indices_info.indices_count,
                }
            },
        };

        let result = Primitive {
            params: render_params,
            material: doc_primitive.material().index(),
        };
        Ok(result)
    }

    pub fn record_command(&self, recorder: &VkCmdRecorder<IGraphics>, model: &VkglTFModel, params: &ModelRenderParams) {

        let material_data = model.materials.list.get(self.material.unwrap_or(DEFAULT_MATERIAL_INDEX));
        recorder.push_constants(params.pipeline_layout, params.material_stage, 0, material_data);

        match self.params {
            | RenderParams::DrawArray { vertex_count, first_vertex } => {
                recorder.draw(vertex_count, 1, first_vertex, 0);
            },
            | RenderParams::DrawIndex {  index_count,  first_index } => {
                recorder.draw_indexed(index_count, 1, first_index, 0, 0);
            },
        }
    }
}
// --------------------------------------------------------------------------------------

// --------------------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub enum RenderParams {
    DrawArray { vertex_count: vkuint, first_vertex: vkuint },
    DrawIndex {  index_count: vkuint,  first_index: vkuint },
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

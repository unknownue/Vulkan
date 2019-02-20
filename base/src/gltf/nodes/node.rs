
use crate::gltf::asset::{ReferenceIndex, AssetElementList};
use crate::gltf::asset::{VkglTFModel, ModelRenderParams};
use crate::gltf::nodes::attachment::{NodeAttachments, AttachmentContent};
use crate::command::{VkCmdRecorder, IGraphics, CmdGraphicsApi};
use crate::error::VkResult;
use crate::vkuint;

type Matrix4F = nalgebra::Matrix4<f32>;

// --------------------------------------------------------------------------------------
/// A wrapper class for node level in glTF, containing the render parameters read from glTF file.
pub struct Node {

    /// the name property of current node.
    name: Option<String>,
    /// the reference to MeshEntity.
    local_mesh: Option<LocalMesh>,
    /// the json index of children nodes.
    children: Vec<ReferenceIndex>,
    /// the transform property of current node.
    local_transform: Matrix4F,
}

struct LocalMesh {
    /// the json index of glTF::Mesh.
    mesh_index: ReferenceIndex,
    /// the drawing order of node hierarchy.
    render_order: usize,
}

impl Node {

    pub fn from_doc(node: gltf::Node, render_counter: usize) -> VkResult<Node> {

        // read the name of Node.
        let name = node.name().and_then(|n| Some(n.to_string()));

        // read the transform matrix of Node.
        let local_transform = Matrix4F::from(node.transform().matrix());

        // first, read the mesh referenced by current node.
        let local_mesh = if let Some(doc_mesh) = node.mesh() {

            let local_mesh = LocalMesh {
                mesh_index  : doc_mesh.index(),
                render_order: render_counter,
            };

            Some(local_mesh)
        } else {
            None
        };

        // and then, read the child nodes of current node recursively.
        let children = node.children()
            .map(|doc_node| doc_node.index())
            .collect();

        let result = Node { name, local_mesh, children, local_transform };
        Ok(result)
    }

    pub fn read_attachment(&self, nodes: &AssetElementList<Node>, attachments: &mut NodeAttachments, parent_transform: &Matrix4F) {

        // apply parent node's transformation to current node level.
        let node_transform: Matrix4F = parent_transform * self.local_transform;

        if self.local_mesh.is_some() {

            let attachment = AttachmentContent {
                transform: Some(node_transform.clone()),
            };
            // read the final attachment data.
            attachments.data_content.extend(attachment);
        }

        // update child nodes recursively.
        for child_json_index in self.children.iter().cloned() {
            let child_node = nodes.get(child_json_index);
            child_node.read_attachment(nodes, attachments, &node_transform);
        }
    }

    pub fn record_command(&self, recorder: &VkCmdRecorder<IGraphics>, model: &VkglTFModel, params: &ModelRenderParams) {

        if let Some(ref local_mesh) = self.local_mesh {

            // calculate the dynamic offset.
            let dyn_offset = (model.nodes.attachment_size_aligned as vkuint) * (local_mesh.render_order as vkuint);
            // bind descriptors with dynamic offset for node attachment.
            recorder.bind_descriptor_sets(params.pipeline_layout, 0, &[params.descriptor_set], &[dyn_offset]);

            let mesh = model.meshes.list.get(local_mesh.mesh_index);
            mesh.record_command(recorder, model, params);
        }

        for child_node_index in self.children.iter().cloned() {
            let child_node = model.nodes.list.get(child_node_index);
            child_node.record_command(recorder, model, params);
        }
    }
}
// --------------------------------------------------------------------------------------

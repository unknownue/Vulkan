
use crate::gltf::asset::{ReferenceIndex, AssetElementList};
use crate::gltf::nodes::attachment::{NodeAttachments, AttachmentContent};
use crate::error::VkResult;

type Matrix4F = nalgebra::Matrix4<f32>;

// --------------------------------------------------------------------------------------
/// A wrapper class for node level in glTF, containing the render parameters read from glTF file.
pub struct Node {

    /// the name property of current node.
    name: Option<String>,
    /// the reference to MeshEntity.
    local_mesh: Option<ReferenceIndex>,
    /// the json index of children nodes.
    children: Vec<ReferenceIndex>,
    /// the transform property of current node.
    local_transform: Matrix4F,
}

impl Node {

    pub fn from_doc(node: gltf::Node) -> VkResult<Node> {

        // read the name of Node.
        let name = node.name().and_then(|n| Some(n.to_string()));

        // read the transform matrix of Node.
        let local_transform = Matrix4F::from(node.transform().matrix());

        // first, read the mesh referenced by current node.
        let local_mesh = if let Some(doc_mesh) = node.mesh() {
            Some(doc_mesh.index())
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
        let node_transform = parent_transform * self.local_transform;

        let attachment = AttachmentContent {
            transform: Some(node_transform.clone()),
        };
        // read the final attachment data.
        attachments.content.extend(attachment);

        // update child nodes recursively.
        for child_json_index in self.children.iter().cloned() {
            let child_node = nodes.get(child_json_index);
            child_node.read_attachment(nodes, attachments, &node_transform);
        }
    }
}
// --------------------------------------------------------------------------------------

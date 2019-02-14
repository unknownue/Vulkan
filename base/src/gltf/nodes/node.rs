
use crate::gltf::asset::ReferenceIndex;
use crate::error::VkResult;

type Matrix4F = nalgebra::Matrix4<f32>;
type NodeIndex = usize;

// --------------------------------------------------------------------------------------
/// A wrapper class for node level in glTF, containing the render parameters read from glTF file.
pub struct Node {

    /// the name property of current node.
    name: Option<String>,
    /// the reference to MeshEntity.
    local_mesh: Option<ReferenceIndex>,
    /// the json index of children nodes.
    children: Vec<NodeIndex>,
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

    pub fn transform(&self) -> &Matrix4F {
        &self.local_transform
    }
//    /// Apply parent node's transformation to current node level.
//    pub fn combine_transform(&mut self, parent_transform: &Matrix4F) {
//        self.local_transform = parent_transform * self.local_transform;
//
//        for child_node in self.children.iter_mut() {
//            child_node.combine_transform(&self.local_transform);
//        }
//    }
}
// --------------------------------------------------------------------------------------

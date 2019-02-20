
use crate::gltf::asset::{ReferenceIndex, AssetElementList};
use crate::gltf::asset::{VkglTFModel, ModelRenderParams};
use crate::gltf::nodes::{Node, NodeAttachments};
use crate::command::{VkCmdRecorder, IGraphics};

type Matrix4F = nalgebra::Matrix4<f32>;


pub struct Scene {

    /// a scene may contain multiple glTF::Node.
    nodes: Vec<ReferenceIndex>,
}

impl Scene {

    pub fn from_doc(doc_scene: gltf::Scene) -> Scene {

        let nodes = doc_scene.nodes()
            .map(|doc_node| doc_node.index())
            .collect();

        Scene { nodes }
    }

    pub fn read_node_attachment(&self, nodes: &AssetElementList<Node>, attachments: &mut NodeAttachments) {

        for node_json_index in self.nodes.iter().cloned() {
            let node = nodes.get(node_json_index);
            node.read_attachment(nodes, attachments, &Matrix4F::identity());
        }
    }

    pub fn record_command(&self, recorder: &VkCmdRecorder<IGraphics>, model: &VkglTFModel, params: &ModelRenderParams) {

        for node_json_index in self.nodes.iter().cloned() {

            let node = model.nodes.list.get(node_json_index);
            node.record_command(recorder, model, params);
        }
    }
}


use crate::gltf::asset::{GltfDocument, AssetAbstract};
use crate::gltf::asset::{ReferenceIndex, StorageIndex};
use crate::gltf::nodes::node::Node;
use crate::gltf::nodes::attachment::{NodeAttachments, NodeAttachmentFlags};
use crate::error::{VkResult, VkTryFrom};

use std::collections::HashMap;

pub struct NodeAsset {

    attachments: NodeAttachments,
    nodes: Vec<Node>,

    query_table: HashMap<ReferenceIndex, StorageIndex>,
}

impl VkTryFrom<NodeAttachmentFlags> for NodeAsset {

    fn try_from(flag: NodeAttachmentFlags) -> VkResult<NodeAsset> {

        let result = NodeAsset {
            attachments: NodeAttachments::try_from(flag)?,
            nodes: Vec::new(),
            query_table: HashMap::new(),
        };
        Ok(result)
    }
}

impl<'a> AssetAbstract<'a> for NodeAsset {
    const ASSET_NAME: &'static str = "Nodes";

    fn read_doc(&mut self, source: &GltfDocument) -> VkResult<()> {

        for doc_node in source.doc.nodes() {

            let json_index = doc_node.index();
            let storage_index = self.nodes.len();
            self.query_table.insert(json_index, storage_index);

            let node = Node::from_doc(doc_node)?;
            self.nodes.push(node);
        }

        Ok(())
    }
}

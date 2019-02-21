
use ash::vk;

use crate::gltf::asset::{GltfDocument, AssetAbstract, AssetElementList};
use crate::gltf::asset::ReferenceIndex;
use crate::gltf::scene::Scene;
use crate::gltf::nodes::node::Node;
use crate::gltf::nodes::attachment::{NodeAttachments, NodeAttachmentFlags};
use crate::error::{VkResult, VkTryFrom};
use crate::context::VkDevice;
use crate::vkbytes;

use std::collections::HashMap;


pub struct NodeAsset {

    attachments: NodeAttachments,

    nodes: AssetElementList<Node>,
}

pub struct NodeResource {

    pub(crate) list: AssetElementList<Node>,
    pub(crate) attachment_size_aligned: vkbytes,
    pub(crate) attachment_mapping: HashMap<ReferenceIndex, usize>,

    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
}

impl VkTryFrom<NodeAttachmentFlags> for NodeAsset {

    fn try_from(flag: NodeAttachmentFlags) -> VkResult<NodeAsset> {

        let result = NodeAsset {
            attachments: NodeAttachments::try_from(flag)?,
            nodes: Default::default(),
        };
        Ok(result)
    }
}

impl AssetAbstract for NodeAsset {
    const ASSET_NAME: &'static str = "Nodes";

    fn read_doc(&mut self, source: &GltfDocument, scene: &Scene) -> VkResult<()> {

        for doc_node in source.doc.nodes() {

            let json_index = doc_node.index();

            let node = Node::from_doc(doc_node)?;
            self.nodes.push(json_index, node);
        }

        scene.read_node_attachment(&self.nodes, &mut self.attachments);

        Ok(())
    }
}

impl NodeAsset {

    pub fn allocate(self, device: &VkDevice) -> VkResult<NodeResource> {

        use crate::ci::buffer::BufferCI;
        use crate::ci::memory::MemoryAI;
        use crate::ci::VkObjectBuildableCI;
        use crate::utils::memory::bound_to_alignment;

        let min_alignment = device.phy.limits.min_uniform_buffer_offset_alignment;
        let attachment_size_aligned = bound_to_alignment(self.attachments.element_size, min_alignment);

        // create dynamic uniform buffer and memory.
        let uniform_size = attachment_size_aligned * (self.attachments.data_content.length() as vkbytes);
        let (uniform_buffer, uniform_requirement) = BufferCI::new(uniform_size)
            .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
            .build(device)?;

        let memory_type = crate::utils::memory::get_memory_type_index(device, uniform_requirement.memory_type_bits, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
        let uniform_memory = MemoryAI::new(uniform_requirement.size, memory_type)
            .build(device)?;

        // map and bind uniform buffer to memory.
        let data_ptr = device.map_memory(uniform_memory, 0, vk::WHOLE_SIZE)?;
        self.attachments.data_content.map_data(data_ptr, uniform_requirement.size, min_alignment);
        device.unmap_memory(uniform_memory);

        // bind vertex buffer to memory.
        device.bind_memory(uniform_buffer, uniform_memory, 0)?;

        let result = NodeResource {
            list  : self.nodes,
            buffer: uniform_buffer,
            memory: uniform_memory,
            attachment_mapping: self.attachments.attachments_mapping,
            attachment_size_aligned,
        };
        Ok(result)
    }
}

impl NodeResource {

    pub fn node_descriptor(&self) -> vk::DescriptorBufferInfo {

        vk::DescriptorBufferInfo {
            buffer: self.buffer,
            offset: 0,
            range : self.attachment_size_aligned,
        }
    }

    pub fn discard(&self, device: &VkDevice) {

        device.discard(self.buffer);
        device.discard(self.memory);
    }
}

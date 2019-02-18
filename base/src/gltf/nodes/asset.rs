
use ash::vk;
use ash::version::DeviceV1_0;

use crate::gltf::asset::{GltfDocument, AssetAbstract, AssetElementList};
use crate::gltf::scene::Scene;
use crate::gltf::nodes::node::Node;
use crate::gltf::nodes::attachment::{NodeAttachments, NodeAttachmentFlags};
use crate::error::{VkResult, VkError, VkTryFrom};
use crate::context::VkDevice;
use crate::vkbytes;

pub struct NodeAsset {

    attachments: NodeAttachments,

    nodes: AssetElementList<Node>,
}

pub struct NodeAssetBlock {
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

    fn allocate(self, device: &VkDevice) -> VkResult<NodeAssetBlock> {

        use crate::ci::buffer::BufferCI;
        use crate::ci::memory::MemoryAI;
        use crate::ci::VkObjectBuildableCI;

        // create dynamic uniform buffer and memory.
        let uniform_size = self.attachments.element_size * (self.attachments.data_content.length() as vkbytes);
        let (uniform_buffer, uniform_requirement) = BufferCI::new(uniform_size)
            .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
            .build(device)?;

        let memory_type = crate::utils::memory::get_memory_type_index(device, uniform_requirement.memory_type_bits, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
        let uniform_memory = MemoryAI::new(uniform_requirement.size, memory_type)
            .build(device)?;

        // map and bind uniform buffer to memory.
        unsafe {

            let min_alignment = device.phy.limits.min_uniform_buffer_offset_alignment;
            // map uniform data.
            let data_ptr = device.logic.handle.map_memory(uniform_memory, 0, uniform_requirement.size, vk::MemoryMapFlags::empty())
                .map_err(|_| VkError::device("Map Memory"))?;
            self.attachments.data_content.map_data(data_ptr, uniform_requirement.size, min_alignment);

            // unmap the memory.
            device.logic.handle.unmap_memory(uniform_memory);
        }

        // bind vertex buffer to memory.
        device.bind(uniform_buffer, uniform_memory, 0)?;

        let result = NodeAssetBlock {
            buffer: uniform_buffer,
            memory: uniform_memory,
        };
        Ok(result)
    }
}

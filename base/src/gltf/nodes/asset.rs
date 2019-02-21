
use ash::vk;

use crate::gltf::asset::{GltfDocument, AssetAbstract, AssetElementList};
use crate::gltf::asset::ReferenceIndex;
use crate::gltf::scene::Scene;
use crate::gltf::nodes::node::Node;
use crate::gltf::nodes::attachment::{NodeAttachments, NodeAttachmentFlags};

use crate::command::{VkCmdRecorder, ITransfer, CmdTransferApi};
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

    pub fn allocate(self, device: &VkDevice, cmd_recorder: &VkCmdRecorder<ITransfer>) -> VkResult<NodeResource> {

        use crate::ci::buffer::BufferCI;
        use crate::ci::memory::MemoryAI;
        use crate::ci::VkObjectBuildableCI;
        use crate::utils::memory::{bound_to_alignment, get_memory_type_index};

        let min_alignment = device.phy.limits.min_uniform_buffer_offset_alignment;
        let attachment_size_aligned = bound_to_alignment(self.attachments.element_size, min_alignment);
        let request_attachments_size = attachment_size_aligned * (self.attachments.data_content.length() as vkbytes);

        // create staging buffer and memory.
        let (staging_buffer, staging_requirement) = BufferCI::new(request_attachments_size)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC)
            .build(device)?;
        let staging_memory = MemoryAI::new(
            staging_requirement.size,
            get_memory_type_index(device, staging_requirement.memory_type_bits, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT))
            .build(device)?;

        // map and bind staging buffer to memory.
        device.bind_memory(staging_buffer, staging_memory, 0)?;
        let data_ptr = device.map_memory(staging_memory, 0, vk::WHOLE_SIZE)?;
        self.attachments.data_content.map_data(data_ptr, staging_requirement.size, min_alignment);
        device.unmap_memory(staging_memory);

        // allocate dynamic uniform buffer and memory for Node attachments data.
        let (attachments_buffer, attachments_requirement) = BufferCI::new(request_attachments_size)
            .usage(vk::BufferUsageFlags::UNIFORM_BUFFER | vk::BufferUsageFlags::TRANSFER_DST)
            .build(device)?;
        let attachments_memory = MemoryAI::new(
            attachments_requirement.size,
            get_memory_type_index(device, attachments_requirement.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL))
            .build(device)?;
        device.bind_memory(attachments_buffer, attachments_memory, 0)?;

        // copy staging data to target memory.
        cmd_recorder.reset_command(vk::CommandBufferResetFlags::empty())?;

        let copy_region = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size: staging_requirement.size,
        };

        cmd_recorder.begin_record()?
            .copy_buf2buf(staging_buffer, attachments_buffer, &[copy_region])
            .end_record()?;

        cmd_recorder.flush_copy_command(device.logic.queues.transfer.handle)?;

        // destroy staging buffer and memory.
        device.discard(staging_buffer);
        device.discard(staging_memory);

        // done.
        let result = NodeResource {
            list  : self.nodes,
            buffer: attachments_buffer,
            memory: attachments_memory,
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

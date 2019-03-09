
use ash::vk;

use crate::gltf::asset::{GltfDocument, AssetAbstract, AssetElementList};
use crate::gltf::asset::ReferenceIndex;
use crate::gltf::scene::Scene;
use crate::gltf::nodes::node::Node;
use crate::gltf::nodes::attachment::{NodeAttachments, NodeAttachmentFlags};

use crate::ci::vma::VmaBuffer;
use crate::context::{VkDevice, VmaResourceDiscardable};
use crate::command::CmdTransferApi;
use crate::error::{VkResult, VkErrorKind, VkTryFrom};
use crate::{vkbytes, vkptr};

use std::collections::HashMap;


pub struct NodeAsset {

    attachments: NodeAttachments,

    nodes: AssetElementList<Node>,
}

pub struct NodeResource {

    pub(crate) list: AssetElementList<Node>,
    pub(crate) attachment_size_aligned: vkbytes,
    pub(crate) attachment_mapping: HashMap<ReferenceIndex, usize>,

    buffer: VmaBuffer,
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

    pub fn allocate(self, device: &mut VkDevice, min_alignment: vkbytes) -> VkResult<NodeResource> {

        use crate::ci::buffer::BufferCI;
        use crate::ci::vma::VmaAllocationCI;
        use crate::utils::memory::IntegerAlignable;

        let attachment_size_aligned = self.attachments.element_size.align_to(min_alignment);
        let request_attachments_size = attachment_size_aligned * (self.attachments.data_content.length() as vkbytes);

        // allocate dynamic uniform buffer for Node attachments data.
        let attachments_buffer = {

            let attachments_ci = BufferCI::new(request_attachments_size)
                .usage(vk::BufferUsageFlags::UNIFORM_BUFFER | vk::BufferUsageFlags::TRANSFER_DST);
            let allocate_ci = VmaAllocationCI::new(vma::MemoryUsage::GpuOnly, vk::MemoryPropertyFlags::DEVICE_LOCAL);
            let attachments_allocation = device.vma.create_buffer(
                &attachments_ci.value(), allocate_ci.as_ref())
                .map_err(VkErrorKind::Vma)?;

            VmaBuffer::from(attachments_allocation)
        };

        // allocate staging buffer.
        let staging_buffer = {

            let staging_ci = BufferCI::new(request_attachments_size)
                .usage(vk::BufferUsageFlags::TRANSFER_SRC);
            let allocate_ci = VmaAllocationCI::new(vma::MemoryUsage::CpuToGpu, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
            let (staging_buffer, allocation, info) = device.vma.create_buffer(
                &staging_ci.value(), &allocate_ci.as_ref())
                .map_err(VkErrorKind::Vma)?;

            let data_ptr = device.vma.map_memory(&allocation)
                .map_err(VkErrorKind::Vma)? as vkptr;

            self.attachments.data_content.map_data(data_ptr, info.get_size() as _, min_alignment);

            device.vma.unmap_memory(&allocation)
                .map_err(VkErrorKind::Vma)?;

            VmaBuffer { handle: staging_buffer, allocation, info }
        };

        { // copy staging data to target memory.
            let cmd_recorder = device.get_transfer_recorder();

            let copy_region = vk::BufferCopy {
                src_offset: 0,
                dst_offset: 0,
                size: staging_buffer.info.get_size() as _,
            };

            cmd_recorder.begin_record()?
                .copy_buf2buf(staging_buffer.handle, attachments_buffer.handle, &[copy_region])
                .end_record()?;

            device.flush_transfer(cmd_recorder)?;
        }

        { // destroy staging buffer.
            device.vma_discard(staging_buffer)?;
        }

        // done.
        let result = NodeResource {
            list  : self.nodes,
            buffer: attachments_buffer,
            attachment_mapping: self.attachments.attachments_mapping,
            attachment_size_aligned,
        };
        Ok(result)
    }
}

impl NodeResource {

    pub fn node_descriptor(&self) -> vk::DescriptorBufferInfo {

        vk::DescriptorBufferInfo {
            buffer: self.buffer.handle,
            offset: 0,
            range : self.attachment_size_aligned,
        }
    }
}

impl VmaResourceDiscardable for NodeResource {

    fn discard_by(self, vma: &mut vma::Allocator) -> VkResult<()> {
        vma.destroy_buffer(self.buffer.handle, &self.buffer.allocation)
            .map_err(VkErrorKind::Vma)?;
        Ok(())
    }
}

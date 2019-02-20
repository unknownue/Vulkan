
use ash::vk;

use crate::gltf::asset::{GltfDocument, AssetAbstract, AssetElementList};
use crate::gltf::scene::Scene;
use crate::gltf::meshes::mesh::Mesh;
use crate::gltf::meshes::attributes::{AttributesData, AttributeFlags};
use crate::gltf::meshes::indices::IndicesData;

use crate::ci::VkObjectBuildableCI;
use crate::ci::pipeline::VertexInputSCI;
use crate::ci::device::SubmitCI;
use crate::ci::memory::MemoryAI;
use crate::ci::sync::FenceCI;

use crate::context::VkDevice;
use crate::command::{VkCmdRecorder, IGraphics, CmdGraphicsApi};
use crate::utils::time::VkTimeDuration;
use crate::utils::memory::{get_memory_type_index, bound_to_alignment};
use crate::error::{VkResult, VkTryFrom};
use crate::vkbytes;


pub struct MeshAsset {

    attributes: AttributesData,
    indices: IndicesData,

    meshes: AssetElementList<Mesh>,
}

struct MeshAssetBlock {

    vertex: BufferBlock,
    index: Option<BufferBlock>,

    memory: vk::DeviceMemory,
}

pub struct MeshResource {

    pub(crate) list: AssetElementList<Mesh>,

    vertex: BufferBlock,
    index : Option<BufferBlock>,
    memory: vk::DeviceMemory,

    pub vertex_input: VertexInputSCI,
}

pub struct BufferBlock {
    pub buffer: vk::Buffer,
    pub size: vkbytes,
}

impl VkTryFrom<AttributeFlags> for MeshAsset {

    fn try_from(flag: AttributeFlags) -> VkResult<MeshAsset> {

        let result = MeshAsset {
            attributes: AttributesData::try_from(flag)?,
            indices: Default::default(),
            meshes : Default::default(),
        };
        Ok(result)
    }
}

impl AssetAbstract for MeshAsset {
    const ASSET_NAME: &'static str = "Meshes";

    fn read_doc(&mut self, source: &GltfDocument, _scene: &Scene) -> VkResult<()> {

        for doc_mesh in source.doc.meshes() {

            let json_index = doc_mesh.index();
            let mesh = Mesh::from_doc(doc_mesh, source, &mut self.attributes, &mut self.indices)?;

            self.meshes.push(json_index, mesh);
        }

        Ok(())
    }
}

impl MeshAsset {

    pub fn allocate(self, device: &VkDevice) -> VkResult<MeshResource> {

        // allocate staging buffer.
        let staging_block = self.allocate_staging(device)?;
        // allocate mesh buffer.
        let mesh_block = self.allocate_mesh(device)?;

        // copy data from staging buffer to mesh buffer.
        MeshAsset::copy_staging2mesh(device, &staging_block, &mesh_block)?;

        // discard staging resource.
        staging_block.discard(device);

        let result = MeshResource {
            vertex: mesh_block.vertex,
            index : mesh_block.index,
            memory: mesh_block.memory,
            list  : self.meshes,
            vertex_input: self.attributes.input_descriptions(),
        };
        Ok(result)
    }

    fn allocate_mesh(&self, device: &VkDevice) -> VkResult<MeshAssetBlock> {

        // create buffer and allocate memory for glTF mesh.
        let (vertex_buffer, vertex_requirement) = self.attributes.buffer_ci()
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST)
            .build(device)?;
        let vertex_aligned_size = bound_to_alignment(vertex_requirement.size, vertex_requirement.alignment);

        let mesh_block = if let Some(indices_ci) = self.indices.buffer_ci() {
            let (index_buffer, index_requirement) = indices_ci
                .usage(vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST)
                .build(device)?;
            let index_aligned_size = bound_to_alignment(index_requirement.size, index_requirement.alignment);

            let memory_type = get_memory_type_index(device, vertex_requirement.memory_type_bits & index_requirement.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL);
            let mesh_memory = MemoryAI::new(vertex_aligned_size + index_aligned_size, memory_type)
                .build(device)?;

            MeshAssetBlock {
                vertex: BufferBlock { buffer: vertex_buffer, size: vertex_aligned_size },
                index: Some(BufferBlock { buffer: index_buffer, size: index_aligned_size }),
                memory: mesh_memory,
            }
        } else {
            let memory_type = get_memory_type_index(device, vertex_requirement.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL);
            let mesh_memory = MemoryAI::new(vertex_requirement.size, memory_type)
                .build(device)?;

            MeshAssetBlock {
                vertex: BufferBlock { buffer: vertex_buffer, size: vertex_requirement.size },
                index: None,
                memory: mesh_memory,
            }
        };

        // bind vertex buffer to memory.
        device.bind_memory(mesh_block.vertex.buffer, mesh_block.memory, 0)?;
        // bind index buffer to memory.
        if let Some(ref index_buffer) = mesh_block.index {
            device.bind_memory(index_buffer.buffer, mesh_block.memory, mesh_block.vertex.size)?;
        }

        Ok(mesh_block)
    }

    fn allocate_staging(&self, device: &VkDevice) -> VkResult<MeshAssetBlock> {

        // create staging buffer and allocate memory.
        let (vertex_buffer, vertex_requirement) = self.attributes.buffer_ci()
            .usage(vk::BufferUsageFlags::TRANSFER_SRC)
            .build(device)?;
        let vertex_aligned_size = bound_to_alignment(vertex_requirement.size, vertex_requirement.alignment);

        let mesh_block = if let Some(indices_ci) = self.indices.buffer_ci() {
            let (index_buffer, index_requirement) = indices_ci
                .usage(vk::BufferUsageFlags::TRANSFER_SRC)
                .build(device)?;
            let index_aligned_size = bound_to_alignment(index_requirement.size, index_requirement.alignment);

            let memory_type = get_memory_type_index(device, vertex_requirement.memory_type_bits | index_requirement.memory_type_bits, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
            let mesh_memory = MemoryAI::new(vertex_aligned_size + index_aligned_size, memory_type)
                .build(device)?;

            MeshAssetBlock {
                vertex: BufferBlock { buffer: vertex_buffer, size: vertex_aligned_size },
                index: Some(BufferBlock { buffer: index_buffer, size: index_aligned_size }),
                memory: mesh_memory,
            }

        } else {
            let memory_type = get_memory_type_index(device, vertex_requirement.memory_type_bits, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
            let mesh_memory = MemoryAI::new(vertex_requirement.size, memory_type)
                .build(device)?;

            MeshAssetBlock {
                vertex: BufferBlock { buffer: vertex_buffer, size: vertex_requirement.size },
                index: None,
                memory: mesh_memory,
            }
        };

        // bind vertex buffer to memory.
        device.bind_memory(mesh_block.vertex.buffer, mesh_block.memory, 0)?;
        // bind index buffer to memory.
        if let Some(ref index_buffer) = mesh_block.index {
            device.bind_memory(index_buffer.buffer, mesh_block.memory, mesh_block.vertex.size)?;
        }

        // map and bind staging buffer to memory.

        if let Some(ref index_buffer) = mesh_block.index {

            // get the starting pointer of host memory.
            let data_ptr = device.map_memory(mesh_block.memory, 0, mesh_block.vertex.size + index_buffer.size)?;
            // map vertex data.
            self.attributes.data_content.map_data(data_ptr);

            let data_ptr = unsafe { data_ptr.offset(mesh_block.vertex.size as _) };
            // map index data.
            self.indices.map_data(data_ptr);
        } else {

            // map vertex data.
            let data_ptr = device.map_memory(mesh_block.memory, 0, mesh_block.vertex.size)?;
            self.attributes.data_content.map_data(data_ptr);
        }

        // unmap the memory.
        device.unmap_memory(mesh_block.memory);

        Ok(mesh_block)
    }

    fn copy_staging2mesh(device: &VkDevice, staging: &MeshAssetBlock, mesh: &MeshAssetBlock) -> VkResult<()> {

        use crate::ci::command::{CommandBufferAI, CommandPoolCI};
        use crate::command::{VkCmdRecorder, ITransfer, CmdTransferApi};

        let command_pool = CommandPoolCI::new(device.logic.queues.transfer.family_index)
            .build(device)?;

        let copy_command = CommandBufferAI::new(command_pool, 1)
            .build(device)?
            .remove(0);

        let cmd_recorder: VkCmdRecorder<ITransfer> = VkCmdRecorder::new(device, copy_command);

        let vertex_copy_region = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size: staging.vertex.size,
        };

        cmd_recorder.begin_record()?
            .copy_buf2buf(staging.vertex.buffer, mesh.vertex.buffer, &[vertex_copy_region]);


        if let Some(ref index_buffer) = staging.index {
            let index_copy_region = vk::BufferCopy {
                src_offset: staging.vertex.size,
                dst_offset: staging.vertex.size,
                size: index_buffer.size,
            };
            cmd_recorder.copy_buf2buf(index_buffer.buffer, mesh.index.as_ref().unwrap().buffer, &[index_copy_region]);
        }

        cmd_recorder.end_record()?;


        let fence = device.build(&FenceCI::new(false))?;

        let submit_ci = SubmitCI::new()
            .add_command(copy_command);
        device.submit(submit_ci, device.logic.queues.transfer.handle, fence)?;
        device.wait(fence, VkTimeDuration::Infinite)?;

        // release temporary resource.
        device.discard(fence);
        // free the command poll will automatically destroy all command buffers created by this pool.
        device.discard(command_pool);

        Ok(())
    }
}

impl MeshAssetBlock {

    fn discard(&self, device: &VkDevice) {

        device.discard(self.vertex.buffer);
        if let Some(ref index_buffer) = self.index {
            device.discard(index_buffer.buffer);
        }
        device.discard(self.memory);
    }
}

impl MeshResource {

    pub fn record_command(&self, recorder: &VkCmdRecorder<IGraphics>) {

        recorder.bind_vertex_buffers(0, &[self.vertex.buffer], &[0]);

        if let Some(ref index_buffer) = self.index {
            recorder.bind_index_buffer(index_buffer.buffer, vk::IndexType::UINT32, 0);
        }
    }

    pub fn discard(&self, device: &VkDevice) {

        device.discard(self.vertex.buffer);
        if let Some(ref index_buffer) = self.index {
            device.discard(index_buffer.buffer);
        }
        device.discard(self.memory);
    }
}

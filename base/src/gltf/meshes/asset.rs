
use ash::vk;

use crate::gltf::asset::{GltfDocument, AssetAbstract, AssetElementList};
use crate::gltf::scene::Scene;
use crate::gltf::meshes::mesh::Mesh;
use crate::gltf::meshes::attributes::{AttributesData, AttributeFlags};
use crate::gltf::meshes::indices::IndicesData;

use crate::ci::buffer::BufferCI;
use crate::ci::vma::{VmaAllocationCI, VmaBuffer};
use crate::ci::pipeline::VertexInputSCI;

use crate::command::VkCmdRecorder;
use crate::command::{IGraphics, CmdGraphicsApi};
use crate::command::{ITransfer, CmdTransferApi};

use crate::error::{VkResult, VkTryFrom, VkErrorKind};
use crate::vkptr;


pub struct MeshAsset {

    attributes: AttributesData,
    indices: IndicesData,

    meshes: AssetElementList<Mesh>,
}

struct MeshAssetBlock {

    vertices: VmaBuffer,
    indices : Option<VmaBuffer>,
}

pub struct MeshResource {

    pub(crate) list: AssetElementList<Mesh>,

    vertices: VmaBuffer,
    indices: Option<VmaBuffer>,

    pub vertex_input: VertexInputSCI,
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

    pub fn allocate(self, vma: &mut vma::Allocator, cmd_recorder: &VkCmdRecorder<ITransfer>) -> VkResult<MeshResource> {

        // allocate staging buffer.
        let staging_block = self.allocate_staging(vma)?;
        // allocate mesh buffer.
        let mesh_block = self.allocate_mesh(vma)?;

        // copy data from staging buffer to mesh buffer.
        MeshAsset::copy_staging2mesh(cmd_recorder, &staging_block, &mesh_block)?;

        // discard staging resource.
        staging_block.discard(vma)?;

        let result = MeshResource {
            vertices: mesh_block.vertices,
            indices: mesh_block.indices,
            list: self.meshes,
            vertex_input: self.attributes.input_descriptions(),
        };
        Ok(result)
    }

    fn allocate_mesh(&self, vma: &mut vma::Allocator) -> VkResult<MeshAssetBlock> {

        // allocate vertices buffer for glTF attributes.
        let vertex_buffer = {

            let vertex_ci = BufferCI::new(self.attributes.buffer_size_estimated())
                .usage(vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST);
            let allocate_ci = VmaAllocationCI::new(vma::MemoryUsage::GpuOnly, vk::MemoryPropertyFlags::DEVICE_LOCAL);
            let vertices_allocation = vma.create_buffer(
                &vertex_ci.value(), allocate_ci.as_ref())
                .map_err(VkErrorKind::Vma)?;

            VmaBuffer::from(vertices_allocation)
        };

        // allocate index buffer for glTF attributes.
        let index_buffer = if let Some(indices_size) = self.indices.buffer_size_estimated() {

            let indices_ci = BufferCI::new(indices_size)
                .usage(vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST);
            let allocate_ci = VmaAllocationCI::new(vma::MemoryUsage::GpuOnly, vk::MemoryPropertyFlags::DEVICE_LOCAL);
            let indices_allocation = vma.create_buffer(
                &indices_ci.value(), allocate_ci.as_ref())
                .map_err(VkErrorKind::Vma)?;

            Some(VmaBuffer::from(indices_allocation))
        } else {
            None
        };

        let mesh_block = MeshAssetBlock {
            vertices: vertex_buffer,
            indices : index_buffer,
        };
        Ok(mesh_block)
    }

    fn allocate_staging(&self, vma: &mut vma::Allocator) -> VkResult<MeshAssetBlock> {

        let staging_vertices = {

            let vertex_ci = BufferCI::new(self.attributes.buffer_size_estimated())
                .usage(vk::BufferUsageFlags::TRANSFER_SRC);
            let allocate_ci = VmaAllocationCI::new(vma::MemoryUsage::CpuToGpu, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
            let (handle, allocation, info) = vma.create_buffer(
                &vertex_ci.value(), allocate_ci.as_ref())
                .map_err(VkErrorKind::Vma)?;

            let data_ptr = vma.map_memory(&allocation)
                .map_err(VkErrorKind::Vma)? as vkptr;

            self.attributes.data_content.map_data(data_ptr);

            vma.unmap_memory(&allocation)
                .map_err(VkErrorKind::Vma)?;

            VmaBuffer { handle, allocation, info }
        };

        // allocate index buffer for glTF attributes.
        let staging_indices = if let Some(indices_size) = self.indices.buffer_size_estimated() {

            let indices_ci = BufferCI::new(indices_size)
                .usage(vk::BufferUsageFlags::TRANSFER_SRC);
            let allocate_ci = VmaAllocationCI::new(vma::MemoryUsage::CpuToGpu, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
            let (handle, allocation, info) = vma.create_buffer(
                &indices_ci.value(), allocate_ci.as_ref())
                .map_err(VkErrorKind::Vma)?;

            let data_ptr = vma.map_memory(&allocation)
                .map_err(VkErrorKind::Vma)? as vkptr;

            self.indices.map_data(data_ptr);

            vma.unmap_memory(&allocation)
                .map_err(VkErrorKind::Vma)?;

            Some(VmaBuffer { handle, allocation, info })
        } else {
            None
        };

        let staging_meshes = MeshAssetBlock {
            vertices: staging_vertices,
            indices : staging_indices,
        };
        Ok(staging_meshes)
    }

    fn copy_staging2mesh(cmd_recorder: &VkCmdRecorder<ITransfer>, staging: &MeshAssetBlock, meshes: &MeshAssetBlock) -> VkResult<()> {

        cmd_recorder.reset_command(vk::CommandBufferResetFlags::empty())?;
        cmd_recorder.begin_record()?;

        let vertex_copy_region = vk::BufferCopy {
            src_offset: 0, // the starting offset of buffer.
            dst_offset: 0,
            size      : staging.vertices.info.get_size() as _,
        };
        // copy vertices data to target buffer.
        cmd_recorder.copy_buf2buf(staging.vertices.handle, meshes.vertices.handle, &[vertex_copy_region]);

        // copy index data to target buffer.
        if let Some(ref staging_index) = staging.indices {
            if let Some(ref meshes_indices) = meshes.indices {
                let index_copy_region = vk::BufferCopy {
                    src_offset: 0,
                    dst_offset: 0,
                    size      : staging_index.info.get_size() as _,
                };
                cmd_recorder.copy_buf2buf(staging_index.handle, meshes_indices.handle, &[index_copy_region]);
            }
        }

        cmd_recorder.end_record()?;
        // execute and wait the copy operation.
        cmd_recorder.flush_copy_command_by_transfer_queue()?;

        Ok(())
    }
}

impl MeshAssetBlock {

    fn discard(&self, vma: &mut vma::Allocator) -> VkResult<()> {

        vma.destroy_buffer(self.vertices.handle, &self.vertices.allocation)
            .map_err(VkErrorKind::Vma)?;

        if let Some(ref indices) = self.indices {

            vma.destroy_buffer(indices.handle, &indices.allocation)
                .map_err(VkErrorKind::Vma)?;
        }

        Ok(())
    }
}

impl MeshResource {

    pub fn record_command(&self, recorder: &VkCmdRecorder<IGraphics>) {

        recorder.bind_vertex_buffers(0, &[self.vertices.handle], &[0]);

        if let Some(ref index_buffer) = self.indices {
            recorder.bind_index_buffer(index_buffer.handle, vk::IndexType::UINT32, 0);
        }
    }

    pub fn discard_by(&self, vma: &mut vma::Allocator) -> VkResult<()> {

        vma.destroy_buffer(self.vertices.handle, &self.vertices.allocation)
            .map_err(VkErrorKind::Vma)?;

        if let Some(ref indices) = self.indices {

            vma.destroy_buffer(indices.handle, &indices.allocation)
                .map_err(VkErrorKind::Vma)?;
        }

        Ok(())
    }
}

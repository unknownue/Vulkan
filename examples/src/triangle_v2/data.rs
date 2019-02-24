
use ash::vk;

use vkbase::context::VkDevice;
use vkbase::ci::VkObjectBuildableCI;
use vkbase::ci::buffer::BufferCI;
use vkbase::ci::memory::MemoryAI;
use vkbase::VkResult;
use vkbase::{vkuint, vkbytes, Matrix4F};

use std::mem;

/// Vertex layout used in this example.
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    position: [f32; 3],
    color   : [f32; 3],
}

pub struct InputDescriptionStaff {
    pub binding   : vk::VertexInputBindingDescription,
    pub attributes: [vk::VertexInputAttributeDescription; 2],
}

impl Vertex {

    pub fn input_description() -> InputDescriptionStaff {

        let input_binding = vk::VertexInputBindingDescription {
            binding: 0,
            stride : mem::size_of::<Vertex>() as _,
            input_rate: vk::VertexInputRate::VERTEX,
        };

        let vertex_input_attributes = [
            vk::VertexInputAttributeDescription {
                location: 0,
                binding : 0,
                format  : vk::Format::R32G32B32_SFLOAT, // three 32 bit signed (SFLOAT) floats (R32 G32 B32).
                offset  : memoffset::offset_of!(Vertex, position) as _,
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding : 0,
                format  : vk::Format::R32G32B32_SFLOAT,
                offset  : memoffset::offset_of!(Vertex, color) as _,
            },
        ];

        InputDescriptionStaff {
            binding   : input_binding,
            attributes : vertex_input_attributes,
        }
    }
}

/// Vertex buffer.
pub struct VertexBuffer {
    pub memory: vk::DeviceMemory,
    pub buffer: vk::Buffer,
}

/// Index Buffer.
pub struct IndexBuffer {
    pub memory: vk::DeviceMemory,
    pub buffer: vk::Buffer,
    pub count: vkuint,
}

/// Uniform buffer block object.
pub struct UniformBuffer {
    pub memory: vk::DeviceMemory,
    pub buffer: vk::Buffer,
    pub descriptor: vk::DescriptorBufferInfo,
}

pub struct DescriptorStaff {
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set: vk::DescriptorSet,

    pub set_layout: vk::DescriptorSetLayout,
    pub pipeline_layout: vk::PipelineLayout,
}

// The uniform data that will be transferred to shader.
//
//	layout(set = 0, binding = 0) uniform UBO {
//		mat4 projectionMatrix;
//		mat4 viewMatrix;
//		mat4 modelMatrix;
//	} ubo;
#[derive(Debug, Clone, Copy)]
pub struct UboVS {
    pub projection: Matrix4F,
    pub view: Matrix4F,
    pub model: Matrix4F,
}

// Prepare vertex buffer and index buffer for an indexed triangle.
pub fn prepare_vertices(device: &VkDevice) -> VkResult<(VertexBuffer, IndexBuffer)> {

    let vertices_data = [
        Vertex { position: [ 1.0,  1.0, 0.0], color: [1.0, 0.0, 0.0] },
        Vertex { position: [-1.0,  1.0, 0.0], color: [0.0, 1.0, 0.0] },
        Vertex { position: [ 0.0, -1.0, 0.0], color: [0.0, 0.0, 1.0] },
    ];
    let vertices = allocate_buffer(device, &vertices_data, vk::BufferUsageFlags::VERTEX_BUFFER)?;

    let indices_data = [0, 1, 2_u32];
    let indices = allocate_buffer(device, &indices_data, vk::BufferUsageFlags::INDEX_BUFFER)?;

    transfer_staging_data(device, &vertices, &indices)?;

    device.discard(vertices.staging_buffer);
    device.discard(vertices.staging_memory);

    device.discard(indices.staging_buffer);
    device.discard(indices.staging_memory);

    let vertex_buffer = VertexBuffer {
        buffer: vertices.target_buffer,
        memory: vertices.target_memory,
    };

    let index_buffer = IndexBuffer {
        buffer: indices.target_buffer,
        memory: indices.target_memory,
        count: indices_data.len() as _,
    };

    Ok((vertex_buffer, index_buffer))
}

fn transfer_staging_data(device: &VkDevice, vertices: &BufferResourceTmp, indices: &BufferResourceTmp) -> VkResult<()> {

    use vkbase::ci::command::{CommandBufferAI, CommandPoolCI};
    use vkbase::command::{VkCmdRecorder, ITransfer, CmdTransferApi};

    let command_pool = CommandPoolCI::new(device.logic.queues.transfer.family_index)
        .build(device)?;

    let copy_command = CommandBufferAI::new(command_pool, 1)
        .build(device)?
        .remove(0);

    let cmd_recorder: VkCmdRecorder<ITransfer> = VkCmdRecorder::new(device, copy_command);

    let vertex_copy_region = vk::BufferCopy {
        src_offset: 0,
        dst_offset: 0,
        size: vertices.buffer_size,
    };
    let index_copy_region = vk::BufferCopy {
        src_offset: 0,
        dst_offset: 0,
        size: indices.buffer_size,
    };

    cmd_recorder.begin_record()?
        .copy_buf2buf(vertices.staging_buffer, vertices.target_buffer, &[vertex_copy_region])
        .copy_buf2buf(indices.staging_buffer, indices.target_buffer, &[index_copy_region])
        .end_record()?;

    cmd_recorder.flush_copy_command(device.logic.queues.transfer.handle)?;

    // free the command poll will automatically destroy all command buffers created by this pool.
    device.discard(command_pool);

    Ok(())
}


struct BufferResourceTmp {

    buffer_size: vkbytes,

    staging_buffer: vk::Buffer,
    staging_memory: vk::DeviceMemory,

    target_buffer: vk::Buffer,
    target_memory: vk::DeviceMemory,
}

fn allocate_buffer<D: Copy>(device: &VkDevice, data: &[D], buffer_usage: vk::BufferUsageFlags) -> VkResult<BufferResourceTmp> {

    let buffer_size = (mem::size_of::<D>() * data.len()) as vkbytes;

    let (staging_buffer, staging_requirement) = BufferCI::new(buffer_size)
        .usage(vk::BufferUsageFlags::TRANSFER_SRC)
        .build(device)?;

    let staging_memory_index = device.get_memory_type(staging_requirement.memory_type_bits,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
    let staging_memory = MemoryAI::new(staging_requirement.size, staging_memory_index)
        .build(device)?;

    let data_ptr = device.map_memory(staging_memory, 0, staging_requirement.size)?;
    device.copy_to_ptr(data_ptr, data);
    device.unmap_memory(staging_memory);

    device.bind_memory(staging_buffer, staging_memory, 0)?;

    let (target_buffer, target_requirement) = BufferCI::new(buffer_size)
        .usage(vk::BufferUsageFlags::TRANSFER_DST | buffer_usage)
        .build(device)?;

    let target_memory_index = device.get_memory_type(target_requirement.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL);
    let target_memory = MemoryAI::new(target_requirement.size, target_memory_index)
        .build(device)?;

    device.bind_memory(target_buffer, target_memory, 0)?;

    let result = BufferResourceTmp { buffer_size, staging_buffer, staging_memory, target_buffer, target_memory };
    Ok(result)
}

pub fn prepare_uniform(device: &VkDevice, dimension: vk::Extent2D) -> VkResult<UniformBuffer> {

    let (uniform_buffer, memory_requirement) = BufferCI::new(mem::size_of::<UboVS>() as vkbytes)
        .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
        .build(device)?;

    let memory_index = device.get_memory_type(memory_requirement.memory_type_bits,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
    let uniform_memory = MemoryAI::new(memory_requirement.size, memory_index)
        .build(device)?;
    device.bind_memory(uniform_buffer, uniform_memory, 0)?;

    let descriptor_info = vk::DescriptorBufferInfo {
        buffer: uniform_buffer,
        offset: 0,
        range: mem::size_of::<UboVS>() as vkbytes,
    };

    let result = UniformBuffer {
        buffer: uniform_buffer,
        memory: uniform_memory,
        descriptor: descriptor_info,
    };

    update_uniform_buffers(device, dimension, &result)?;

    Ok(result)
}

fn update_uniform_buffers(device: &VkDevice, dimension: vk::Extent2D, uniforms: &UniformBuffer) -> VkResult<()> {

    let screen_aspect = (dimension.width as f32) / (dimension.height as f32);

    let ubo_data = [
        UboVS {
            projection: Matrix4F::new_perspective(screen_aspect, 60.0_f32.to_radians(), 0.1, 256.0),
            view: Matrix4F::new_translation(&nalgebra::Vector3::new(0.0, 0.0, -2.5)),
            model: Matrix4F::identity(),
        },
    ];

    // Map uniform buffer and update it.
    let data_ptr = device.map_memory(uniforms.memory, 0, mem::size_of::<UboVS>() as _)?;
    device.copy_to_ptr(data_ptr, &ubo_data);
    device.unmap_memory(uniforms.memory);

    Ok(())
}


use lazy_static::lazy_static;

use ash::vk;

use std::mem;
use std::ptr;

use vkbase::ci::buffer::BufferCI;
use vkbase::ci::pipeline::VertexInputSCI;
use vkbase::ci::vma::{VmaBuffer, VmaAllocationCI};

use vkbase::context::VkDevice;
use vkbase::FlightCamera;

use vkbase::{vkuint, vkbytes, vkptr, Vec3F, Mat4F};
use vkbase::{VkResult, VkErrorKind};

pub const OBJECT_INSTANCES: usize = 125;

lazy_static! {

    pub static ref VERTEX_DATA: [Vertex; 8] = [
        Vertex { position: Vec3F::new(-1.0, -1.0,  1.0), color: Vec3F::new(1.0, 0.0, 0.0), }, // v0
        Vertex { position: Vec3F::new( 1.0, -1.0,  1.0), color: Vec3F::new(0.0, 1.0, 0.0), }, // v1
        Vertex { position: Vec3F::new( 1.0,  1.0,  1.0), color: Vec3F::new(0.0, 0.0, 1.0), }, // v2
        Vertex { position: Vec3F::new(-1.0,  1.0,  1.0), color: Vec3F::new(0.0, 0.0, 0.0), }, // v3
        Vertex { position: Vec3F::new(-1.0, -1.0, -1.0), color: Vec3F::new(1.0, 0.0, 0.0), }, // v4
        Vertex { position: Vec3F::new( 1.0, -1.0, -1.0), color: Vec3F::new(0.0, 1.0, 0.0), }, // v5
        Vertex { position: Vec3F::new( 1.0,  1.0, -1.0), color: Vec3F::new(0.0, 0.0, 1.0), }, // v6
        Vertex { position: Vec3F::new(-1.0,  1.0, -1.0), color: Vec3F::new(0.0, 0.0, 0.0), }, // v7
    ];

    pub static ref INDEX_DATA: [vkuint; 36] = [
        0,1,2, 2,3,0, 1,5,6, 6,2,1, 7,6,5, 5,4,7, 4,0,3, 3,7,4, 4,5,1, 1,0,4, 3,2,6, 6,7,3,
    ];
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    position: Vec3F,
    color   : Vec3F,
}

impl Vertex {

    pub fn input_description() -> VertexInputSCI {

        VertexInputSCI::new()
            .add_binding(vk::VertexInputBindingDescription {
                binding: 0,
                stride : ::std::mem::size_of::<Vertex>() as _,
                input_rate: vk::VertexInputRate::VERTEX,
            })
            .add_attribute(vk::VertexInputAttributeDescription {
                location: 0,
                binding : 0,
                format  : vk::Format::R32G32B32_SFLOAT, // three 32 bit signed (SFLOAT) floats (R32 G32 B32).
                offset  : memoffset::offset_of!(Vertex, position) as _,
            })
            .add_attribute(vk::VertexInputAttributeDescription {
                location: 1,
                binding : 0,
                format  : vk::Format::R32G32B32_SFLOAT,
                offset  : memoffset::offset_of!(Vertex, color) as _,
            })
    }
}

pub fn generate_cube(device: &mut VkDevice) -> VkResult<(VmaBuffer, VmaBuffer)> {

    // For the sake of simplicity we won't stage the vertex data to the gpu memory.
    let vertex_buffer = {

        let vertices_ci = BufferCI::new((mem::size_of::<Vertex>() * VERTEX_DATA.len()) as vkbytes)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER);
        let allocation_ci = VmaAllocationCI::new(vma::MemoryUsage::CpuOnly, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)
            .flags(vma::AllocationCreateFlags::MAPPED);
        let vertices_allocation = device.vma.create_buffer(
            vertices_ci.as_ref(), allocation_ci.as_ref())
            .map_err(VkErrorKind::Vma)?;

        unsafe {
            let data_ptr = vertices_allocation.2.get_mapped_data() as vkptr<Vertex>;
            debug_assert_ne!(data_ptr, ptr::null_mut());
            data_ptr.copy_from_nonoverlapping(VERTEX_DATA.as_ptr(), VERTEX_DATA.len());
        }

        VmaBuffer::from(vertices_allocation)
    };

    let index_buffer = {

        let indices_ci = BufferCI::new((mem::size_of::<vkuint>() * INDEX_DATA.len()) as vkbytes)
            .usage(vk::BufferUsageFlags::INDEX_BUFFER);
        let allocation_ci = VmaAllocationCI::new(vma::MemoryUsage::CpuOnly, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)
            .flags(vma::AllocationCreateFlags::MAPPED);
        let indices_allocation = device.vma.create_buffer(
            indices_ci.as_ref(), allocation_ci.as_ref())
            .map_err(VkErrorKind::Vma)?;

        unsafe {
            let data_ptr = indices_allocation.2.get_mapped_data() as vkptr<vkuint>;
            debug_assert_ne!(data_ptr, ptr::null_mut());
            data_ptr.copy_from_nonoverlapping(INDEX_DATA.as_ptr(), INDEX_DATA.len());
        }

        VmaBuffer::from(indices_allocation)
    };

    Ok((vertex_buffer, index_buffer))
}


#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct UboView {
    pub projection: Mat4F,
    pub view      : Mat4F,
}

impl UboView {

    pub fn prepare_buffer(device: &mut VkDevice, camera: &FlightCamera) -> VkResult<(VmaBuffer, UboView)> {

        let buffer_ci = BufferCI::new(mem::size_of::<UboView>() as vkbytes)
            .usage(vk::BufferUsageFlags::UNIFORM_BUFFER);
        let allocation_ci = VmaAllocationCI::new(vma::MemoryUsage::CpuOnly, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)
            .flags(vma::AllocationCreateFlags::MAPPED);
        let buffer_allocation = device.vma.create_buffer(
            buffer_ci.as_ref(), allocation_ci.as_ref())
            .map_err(VkErrorKind::Vma)?;

        let ubo_view_data = UboView {
            projection: camera.proj_matrix(),
            view      : camera.view_matrix(),
        };

        unsafe {
            let data_ptr = buffer_allocation.2.get_mapped_data() as vkptr<UboView>;
            debug_assert_ne!(data_ptr, ptr::null_mut());
            data_ptr.copy_from_nonoverlapping(&ubo_view_data, 1);
        }

        Ok((VmaBuffer::from(buffer_allocation), ubo_view_data))
    }
}

pub struct UboDynamicData {
    pub model: [Mat4F; OBJECT_INSTANCES],
}

impl UboDynamicData {

    fn identity() -> UboDynamicData {
        UboDynamicData {
            model: [Mat4F::identity(); OBJECT_INSTANCES],
        }
    }

    pub fn prepare_buffer(device: &mut VkDevice) -> VkResult<(VmaBuffer, UboDynamicData, vkuint)> {

        let min_alignment = device.phy.limits.min_uniform_buffer_offset_alignment as usize;
        println!("minUniformBufferOffsetAlignment in Vulkan: {}", min_alignment);

        // Calculate required alignment based on minimum device offset alignment.
        let dynamic_alignment = (::std::mem::size_of::<Mat4F>() + min_alignment - 1) & !(min_alignment - 1);
        println!("dynamicAlignment: {}", dynamic_alignment);

        let buffer_ci = BufferCI::new((dynamic_alignment * OBJECT_INSTANCES) as vkbytes)
            .usage(vk::BufferUsageFlags::UNIFORM_BUFFER);
        let allocation_ci = VmaAllocationCI::new(vma::MemoryUsage::CpuOnly, vk::MemoryPropertyFlags::HOST_VISIBLE)
            .flags(vma::AllocationCreateFlags::MAPPED);
        let buffer_allocation = device.vma.create_buffer(
            buffer_ci.as_ref(), allocation_ci.as_ref())
            .map_err(VkErrorKind::Vma)?;

        let data_ptr = buffer_allocation.2.get_mapped_data() as vkptr;
        let initial_data = UboDynamicData::identity();
        debug_assert_ne!(data_ptr, ptr::null_mut());

        let mut data_ptr_aligned = unsafe {
            ash::util::Align::new(data_ptr, dynamic_alignment as _, buffer_allocation.2.get_size() as _)
        };
        data_ptr_aligned.copy_from_slice(&initial_data.model);

        device.vma.flush_allocation(&buffer_allocation.1, 0, vk::WHOLE_SIZE as _)
            .map_err(VkErrorKind::Vma)?;

        Ok((VmaBuffer::from(buffer_allocation), initial_data, dynamic_alignment as vkuint))
    }

    // Although the rotation effect is different from the original implementation..
    pub fn update(&mut self, rotations: &mut RotationData, delta_time: f32) {

        // Dynamic ubo with per-object model matrices indexed by offsets in the command buffer
        let dim: usize = (OBJECT_INSTANCES as f32).powf(1.0 / 3.0) as usize;
        let offset = Vec3F::new(5.0, 5.0, 5.0);

        for x in 0..dim {
            for y in 0..dim {
                for z in 0..dim {

                    let dim_f = dim as f32;

                    let index = x * dim * dim + y * dim + z;
                    // update rotations
                    rotations.rotations[index] += delta_time;

                    let pos = Vec3F::new(
                        -((dim_f * offset.x) / 2.0) + offset.x / 2.0 + (x as f32) * offset.x,
                        -((dim_f * offset.y) / 2.0) + offset.y / 2.0 + (y as f32) * offset.y,
                        -((dim_f * offset.z) / 2.0) + offset.z / 2.0 + (z as f32) * offset.z,
                    );

                    self.model[index] = Mat4F::rotation_3d(rotations.rotations[index] * 2.5, rotations.rotate_speeds[index])
                        .translated_3d(pos);
                }
            }
        }
    }
}


pub struct RotationData {
    pub rotations    : [f32; OBJECT_INSTANCES], // angle
    pub rotate_speeds: [Vec3F; OBJECT_INSTANCES],
}

impl RotationData {

    pub fn new_by_rng() -> RotationData {

        let mut data = RotationData {
            rotations    : [0.0; OBJECT_INSTANCES],
            rotate_speeds: [Vec3F::zero(); OBJECT_INSTANCES],
        };

        use rand::distributions::Distribution;
        let rnd_dist = rand::distributions::Uniform::from(-1.0..1.0_f32);
        let mut rnd_engine = rand::thread_rng();

        for i in 0..OBJECT_INSTANCES {
            data.rotations[i] = rnd_dist.sample(&mut rnd_engine);
            data.rotate_speeds[i] = Vec3F::new(
                rnd_dist.sample(&mut rnd_engine), // generate a random float between -1.0 ~ 1.0.
                rnd_dist.sample(&mut rnd_engine),
                rnd_dist.sample(&mut rnd_engine),
            );
        }

        data
    }
}


use ash::vk;
use ash::version::DeviceV1_0;

use vkbase::context::{VkDevice, VkSwapchain};
use vkbase::ci::VkObjectBuildableCI;
use vkbase::ci::buffer::BufferCI;
use vkbase::ci::memory::MemoryAI;
use vkbase::utils::memory::get_memory_type_index;
use vkbase::utils::time::VkTimeDuration;
use vkbase::gltf::VkglTFModel;
use vkbase::context::VulkanContext;
use vkbase::{FrameAction, vkbytes};
use vkbase::{VkResult, VkError};

use std::ptr;
use std::mem;
use std::path::Path;

use vkexamples::VkExampleBackendRes;
type Matrix4F = nalgebra::Matrix4<f32>;
type Vector4F = nalgebra::Vector4<f32>;

const PHONG_VERTEX_SHADER_SOURCE_PATH      : &'static str = "src/pipelines/phong.vert.glsl";
const PHONG_FRAGMENT_SHADER_SOURCE_PATH    : &'static str = "src/pipelines/phong.frag.glsl";
const TOON_VERTEX_SHADER_SOURCE_PATH       : &'static str = "src/pipelines/toon.vert.glsl";
const TOON_FRAGMENT_SHADER_SOURCE_PATH     : &'static str = "src/pipelines/toon.frag.glsl";
const WIREFRAME_VERTEX_SHADER_SOURCE_PATH  : &'static str = "src/pipelines/wireframe.vert.glsl";
const WIREFRAME_FRAGMENT_SHADER_SOURCE_PATH: &'static str = "src/pipelines/wireframe.frag.glsl";
const MODEL_PATH: &'static str = "models/treasure_smooth.gltf";


pub struct VulkanExample {

    backend_res: VkExampleBackendRes,

    model: VkglTFModel,
    uniform_buffer: UniformBuffer,

    pipelines: PipelineStaff,
    descriptors: DescriptorStaff,
}

struct PipelineStaff {
    phong     : vk::Pipeline,
    wireframe : vk::Pipeline,
    toon      : vk::Pipeline,

    render_pass: vk::RenderPass,
    layout: vk::PipelineLayout,
}

impl VulkanExample {

    pub fn new(context: &VulkanContext) -> VkResult<VulkanExample> {

        let device = &context.device;
        let swapchain = &context.swapchain;
        let dimension = swapchain.dimension;

        let mut backend_res = VkExampleBackendRes::new(device, swapchain)?;

        let (vertex_buffer, index_buffer) = super::data::prepare_model(device)?;
        let uniform_buffer = super::data::prepare_uniform(device, dimension)?;

        let descriptors = setup_descriptor(device, &uniform_buffer)?;

        let render_pass = setup_renderpass(device, &context.swapchain)?;

        backend_res.setup_framebuffers(device, swapchain, render_pass)?;

        let pipeline = prepare_pipelines(device, render_pass, descriptors.pipeline_layout)?;

        let target = VulkanExample {
            backend_res, descriptors,
            pipeline, render_pass,
            vertex_buffer, index_buffer, uniform_buffer,
        };
        Ok(target)
    }
}

impl vkbase::Workflow for VulkanExample {

    fn init(&mut self, device: &VkDevice) -> VkResult<()> {

        self.record_commands(device, self.backend_res.dimension)?;
        Ok(())
    }

    fn render_frame(&mut self, device: &VkDevice, device_available: vk::Fence, await_present: vk::Semaphore, image_index: usize, _delta_time: f32) -> VkResult<vk::Semaphore> {

        let submit_ci = vkbase::ci::device::SubmitCI::new()
            .add_wait(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, await_present)
            .add_command(self.backend_res.commands[image_index])
            .add_signal(self.backend_res.await_rendering);

        // Submit to the graphics queue passing a wait fence.
        device.submit(submit_ci, device.logic.queues.graphics.handle, device_available)?;

        Ok(self.backend_res.await_rendering)
    }

    fn swapchain_reload(&mut self, device: &VkDevice, new_chain: &VkSwapchain) -> VkResult<()> {

        // recreate the resources.
        device.discard(self.pipeline);
        device.discard(self.render_pass);

        self.render_pass = setup_renderpass(device, new_chain)?;
        self.backend_res.swapchain_reload(device, new_chain, self.render_pass)?;
        self.pipeline = prepare_pipelines(device, self.render_pass, self.descriptors.pipeline_layout)?;

        self.record_commands(device, self.backend_res.dimension)?;

        Ok(())
    }

    fn receive_input(&mut self, inputer: &vkbase::InputController, _delta_time: f32) -> FrameAction {

        if inputer.key.is_key_pressed(winit::VirtualKeyCode::Escape) {
            return FrameAction::Terminal
        }

        FrameAction::Rendering
    }

    fn deinit(&mut self, device: &VkDevice) -> VkResult<()> {

        self.discard(device);
        Ok(())
    }
}

impl VulkanExample {

    fn record_commands(&self, device: &VkDevice, dimension: vk::Extent2D) -> VkResult<()> {

        let clear_values = [
            vk::ClearValue { color: vk::ClearColorValue { float32: [0.0, 0.0, 0.2, 1.0] } },
            vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 } },
        ];

        let viewport = vk::Viewport {
            x: 0.0, y: 0.0,
            width: dimension.width as f32, height: dimension.height as f32,
            min_depth: 0.0, max_depth: 1.0,
        };

        let scissor = vk::Rect2D {
            extent: dimension.clone(),
            offset: vk::Offset2D { x: 0, y: 0 },
        };

        for (i, &command) in self.backend_res.commands.iter().enumerate() {

            use vkbase::command::{VkCmdRecorder, CmdGraphicsApi, IGraphics};
            use vkbase::ci::pipeline::RenderPassBI;

            let recorder: VkCmdRecorder<IGraphics> = VkCmdRecorder::new(device, command);

            let render_pass_bi = RenderPassBI::new(self.render_pass, self.backend_res.framebuffers[i])
                .render_extent(dimension)
                .clear_values(&clear_values);

            recorder.begin_record()?
                .begin_render_pass(render_pass_bi)
                .set_viewport(0, &[viewport])
                .set_scissor(0, &[scissor])
                .bind_descriptor_sets(self.descriptors.pipeline_layout, 0, &[self.descriptors.descriptor_set], &[])
                .bind_pipeline(self.pipeline)
                .bind_vertex_buffers(0, &[self.vertex_buffer.buffer], &[0])
                .bind_index_buffer(self.index_buffer.buffer, vk::IndexType::UINT32, 0)
                .draw_indexed(self.index_buffer.count, 1, 0, 0, 1)
                .end_render_pass()
                .end_record()?;
        }

        Ok(())
    }

    fn discard(&self, device: &VkDevice) {

        device.discard(self.descriptors.layout);
        device.discard(self.descriptors.pool);

        device.discard(self.pipelines.phong);
        device.discard(self.pipelines.toon);
        device.discard(self.pipelines.wireframe);
        device.discard(self.pipelines.render_pass);
        device.discard(self.pipelines.layout);

        device.discard(self.uniform_buffer.buffer);
        device.discard(self.uniform_buffer.memory);

        self.backend_res.discard(device);
    }
}

// Prepare model from glTF file.
pub fn prepare_model(device: &VkDevice) -> VkResult<VkglTFModel> {

    use vkbase::gltf::{GltfModelInfo, load_gltf};
    use vkbase::gltf::{AttributeFlags, NodeAttachmentFlags};

    let model_info = GltfModelInfo {
        path: Path::new(MODEL_PATH),
        attribute: AttributeFlags::POSITION | AttributeFlags::NORMAL, // specify model's vertex layout.
        node: NodeAttachmentFlags::TRANSFORM_MATRIX, // specify model's node attachment layout.
    };

    let model = load_gltf(device, model_info)?;
    Ok(model)
}


/// Uniform buffer block object.
pub struct UniformBuffer {
    pub memory: vk::DeviceMemory,
    pub buffer: vk::Buffer,
    pub descriptor: vk::DescriptorBufferInfo,
}

// The uniform data that will be transferred to shader.
//
// layout (set = 0, binding = 0) uniform UBO {
//     mat4 projection;
//     mat4 view;
//     mat4 model;
//     mat4 y_correction;
//     vec4 lightPos;
// } ubo;
#[derive(Debug, Clone, Copy)]
struct UboVS {
    projection   : Matrix4F,
    view         : Matrix4F,
    model        : Matrix4F,
    y_correction : Matrix4F,
    light_pos    : Vector4F,
}

fn prepare_uniform(device: &VkDevice, dimension: vk::Extent2D) -> VkResult<UniformBuffer> {

    let (uniform_buffer, memory_requirement) = BufferCI::new(mem::size_of::<UboVS>() as vkbytes)
        .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
        .build(device)?;

    let memory_index = get_memory_type_index(device, memory_requirement.memory_type_bits, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
    let uniform_memory = MemoryAI::new(memory_requirement.size, memory_index)
        .build(device)?;
    device.bind_memory(uniform_buffer, uniform_memory, 0)?;

    let uniforms = UniformBuffer {
        buffer: uniform_buffer,
        memory: uniform_memory,
        descriptor: vk::DescriptorBufferInfo {
            buffer: uniform_buffer,
            offset: 0,
            range: mem::size_of::<UboVS>() as vkbytes,
        },
    };

    update_uniform_buffers(device, dimension, &uniforms)?;

    Ok(uniforms)
}

fn update_uniform_buffers(device: &VkDevice, dimension: vk::Extent2D, uniforms: &UniformBuffer) -> VkResult<()> {

    let screen_aspect = (dimension.width as f32) / (dimension.height as f32);

    let ubo_data = [
        UboVS {
            projection   : Matrix4F::new_perspective(screen_aspect, 60.0_f32.to_radians(), 0.1, 256.0),
            view         : Matrix4F::new_translation(&nalgebra::Vector3::new(0.0, 0.0, -2.5)),
            model        : Matrix4F::identity(),
            y_correction : vkexamples::Y_CORRECTION.clone(),
            light_pos    : Vector4F::new(0.0, 2.0, 1.0, 0.0),
        },
    ];

    // Map uniform buffer and update it.
    unsafe {
        let data_ptr = device.logic.handle.map_memory(uniforms.memory, 0, mem::size_of::<UboVS>() as vkbytes, vk::MemoryMapFlags::empty())
            .map_err(|_| VkError::device("Map Memory"))?;

        let mapped_copy_target = ::std::slice::from_raw_parts_mut(data_ptr as *mut UboVS, ubo_data.len());
        mapped_copy_target.copy_from_slice(&ubo_data);

        device.logic.handle.unmap_memory(uniforms.memory);
    }

    Ok(())
}

struct DescriptorStaff {
    pool   : vk::DescriptorPool,
    set    : vk::DescriptorSet,
    layout : vk::DescriptorSetLayout,
}

fn setup_descriptor(device: &VkDevice, uniforms: &UniformBuffer) -> VkResult<DescriptorStaff> {

    use vkbase::ci::descriptor::{DescriptorPoolCI, DescriptorSetLayoutCI};
    use vkbase::ci::descriptor::{DescriptorSetAI, DescriptorBufferSetWI};
    use vkbase::ci::pipeline::PipelineLayoutCI;

    // Descriptor Pool.
    let descriptor_pool = DescriptorPoolCI::new(1)
        .add_descriptor(vk::DescriptorType::UNIFORM_BUFFER, 1)
        .add_descriptor(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC, 1)
        .build(device)?;

    // ubo_descriptor represent shader codes as follows:
    // layout (set = 0, binding = 0) uniform UBO {
    //     mat4 projection;
    //     mat4 view;
    //     mat4 model;
    //     mat4 y_correction;
    //     vec4 lightPos;
    // } ubo;
    let ubo_descriptor = vk::DescriptorSetLayoutBinding {
        binding: 0,
        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
        descriptor_count: 1,
        stage_flags: vk::ShaderStageFlags::VERTEX,
        p_immutable_samplers: ptr::null(),
    };
    // node_descriptor represent shader codes as follows:
    // layout (set = 0, binding = 1) uniform NodeAttachments {
    //     mat4 transform;
    // } node_attachments;
    let node_descriptor = vk::DescriptorSetLayoutBinding {
        binding: 1,
        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
        descriptor_count: 1,
        stage_flags: vk::ShaderStageFlags::VERTEX,
        p_immutable_samplers: ptr::null(),
    };

    let set_layout = DescriptorSetLayoutCI::new()
        .add_binding(ubo_descriptor)
        .add_binding(node_descriptor)
        .build(device)?;

    // Descriptor set.
    let mut descriptor_sets = DescriptorSetAI::new(descriptor_pool)
        .add_set_layout(set_layout)
        .build(device)?;
    let descriptor_set = descriptor_sets.remove(0);

    let ubo_write_info = DescriptorBufferSetWI::new(descriptor_set, 0, vk::DescriptorType::UNIFORM_BUFFER)
        .add_buffer(uniforms.descriptor.clone());
    let node_write_info = DescriptorBufferSetWI::new(descriptor_set, 1, vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
        .add_buffer();

    unsafe {
        device.logic.handle.update_descriptor_sets(&[ubo_write_info.value(), node_write_info.value()], &[]);
    }

    let descriptors = DescriptorStaff {
        pool   : descriptor_pool,
        set    : descriptor_set,
        layout : set_layout,
    };
    Ok(descriptors)
}

fn setup_renderpass(device: &VkDevice, swapchain: &VkSwapchain) -> VkResult<vk::RenderPass> {

    use vkbase::ci::pipeline::RenderPassCI;
    use vkbase::ci::pipeline::{AttachmentDescCI, SubpassDescCI, SubpassDependencyCI};

    let color_attachment = AttachmentDescCI::new(swapchain.backend_format)
        .op(vk::AttachmentLoadOp::CLEAR, vk::AttachmentStoreOp::STORE)
        .layout(vk::ImageLayout::UNDEFINED, vk::ImageLayout::PRESENT_SRC_KHR);

    let depth_attachment = AttachmentDescCI::new(device.phy.depth_format)
        .op(vk::AttachmentLoadOp::CLEAR, vk::AttachmentStoreOp::DONT_CARE)
        .layout(vk::ImageLayout::UNDEFINED, vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

    let subpass_description = SubpassDescCI::new(vk::PipelineBindPoint::GRAPHICS)
        .add_color_attachment(0, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL) // Attachment 0 is color.
        .set_depth_stencil_attachment(1, vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL); // Attachment 1 is depth-stencil.

    let dependency0 = SubpassDependencyCI::new(vk::SUBPASS_EXTERNAL, 0)
        .stage_mask(vk::PipelineStageFlags::BOTTOM_OF_PIPE, vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .access_mask(vk::AccessFlags::MEMORY_READ, vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
        .flags(vk::DependencyFlags::BY_REGION);

    let dependency1 = SubpassDependencyCI::new(0, vk::SUBPASS_EXTERNAL)
        .stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, vk::PipelineStageFlags::BOTTOM_OF_PIPE)
        .access_mask(vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE, vk::AccessFlags::MEMORY_READ)
        .flags(vk::DependencyFlags::BY_REGION);

    let render_pass = RenderPassCI::new()
        .add_attachment(color_attachment.value())
        .add_attachment(depth_attachment.value())
        .add_subpass(subpass_description.value())
        .add_dependency(dependency0.value())
        .add_dependency(dependency1.value())
        .build(device)?;

    Ok(render_pass)
}

fn prepare_pipelines(device: &VkDevice, render_pass: vk::RenderPass, set_layout: vk::DescriptorSetLayout) -> VkResult<PipelineStaff> {

    use vkbase::ci::pipeline::*;

    let viewport_state = ViewportSCI::new()
        .add_viewport(vk::Viewport::default())
        .add_scissor(vk::Rect2D::default());

    let rasterization_state = RasterizationSCI::new()
        .polygon(vk::PolygonMode::FILL)
        .cull_face(vk::CullModeFlags::BACK, vk::FrontFace::COUNTER_CLOCKWISE);

    let blend_attachment = BlendAttachmentSCI::new().value();
    let blend_state = ColorBlendSCI::new()
        .add_attachment(blend_attachment);

    let depth_stencil_state = DepthStencilSCI::new()
        .depth_test(true, true, vk::CompareOp::LESS_OR_EQUAL);

    let mut dynamic_state = DynamicSCI::new()
        .add_dynamic(vk::DynamicState::VIEWPORT)
        .add_dynamic(vk::DynamicState::SCISSOR);

    if device.phy.enable_features().wide_lines == vk::TRUE {
        dynamic_state = dynamic_state.add_dynamic(vk::DynamicState::LINE_WIDTH)
    };


    let inputs = Vertex::input_description();
    let vertex_input_state = VertexInputSCI::new()
        .add_binding(inputs.binding)
        .add_attribute(inputs.attributes[0])
        .add_attribute(inputs.attributes[1]);

    // shaders
    use vkbase::ci::shader::{ShaderModuleCI, ShaderStageCI};

    let mut shader_compiler = vkbase::utils::shaderc::VkShaderCompiler::new()?;
    let vert_codes = shader_compiler.compile_from_path(Path::new(PHONG_VERTEX_SHADER_SOURCE_PATH), shaderc::ShaderKind::Vertex, "[Vertex Shader]", "main")?;
    let frag_codes = shader_compiler.compile_from_path(Path::new(PHONG_FRAGMENT_SHADER_SOURCE_PATH), shaderc::ShaderKind::Fragment, "[Fragment Shader]", "main")?;

    let vert_module = ShaderModuleCI::from_glsl(vk::ShaderStageFlags::VERTEX, vert_codes)
        .build(device)?;
    let frag_module = ShaderModuleCI::from_glsl(vk::ShaderStageFlags::FRAGMENT, frag_codes)
        .build(device)?;

    // Pipeline Layout.
    let pipeline_layout = PipelineLayoutCI::new()
        .add_set_layout(set_layout)
        .build(device)?;

    let mut pipeline_ci = GraphicsPipelineCI::new(render_pass, layout);

    pipeline_ci.add_shader_stage(ShaderStageCI::new(vk::ShaderStageFlags::VERTEX, vert_module));
    pipeline_ci.add_shader_stage(ShaderStageCI::new(vk::ShaderStageFlags::FRAGMENT, frag_module));

    pipeline_ci.set_vertex_input(vertex_input_state);
    pipeline_ci.set_viewport(viewport_state);
    pipeline_ci.set_rasterization(rasterization_state);
    pipeline_ci.set_depth_stencil(depth_stencil_state);
    pipeline_ci.set_color_blend(blend_state);
    pipeline_ci.set_dynamic(dynamic_state);

    let pipeline = device.build(&pipeline_ci)?;


    device.discard(vert_module);
    device.discard(frag_module);

    Ok(pipeline)
}

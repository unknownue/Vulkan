
use ash::vk;

use std::ptr;
use std::path::Path;

use vkbase::context::{VkDevice, VkSwapchain};
use vkbase::ci::VkObjectBuildableCI;
use vkbase::VkResult;
use vkbase::FrameAction;

use vkexamples::VkExampleBackend;
use crate::data::{Vertex, VertexBuffer, IndexBuffer, UniformBuffer, DescriptorStaff};

const SHADER_VERTEX_PATH  : &'static str = "examples/src/triangle_v1/triangle.vert.glsl";
const SHADER_FRAGMENT_PATH: &'static str = "examples/src/triangle_v1/triangle.frag.glsl";

pub struct VulkanExample {

    backend: VkExampleBackend,

    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,
    uniform_buffer: UniformBuffer,

    pipeline: vk::Pipeline,

    descriptors: DescriptorStaff,
}

impl VulkanExample {

    pub fn new(context: &mut vkbase::context::VulkanContext) -> VkResult<VulkanExample> {

        let device = &mut context.device;
        let swapchain = &context.swapchain;
        let dimension = swapchain.dimension;

        let render_pass = setup_renderpass(device, &context.swapchain)?;
        let backend_res = VkExampleBackend::new(device, swapchain, render_pass)?;

        let (vertex_buffer, index_buffer) = super::data::prepare_vertices(device)?;
        let uniform_buffer = super::data::prepare_uniform(device, dimension)?;

        let descriptors = setup_descriptor(device, &uniform_buffer)?;

        let pipeline = prepare_pipelines(device, backend_res.render_pass, descriptors.pipeline_layout)?;

        let target = VulkanExample {
            backend: backend_res, descriptors, pipeline,
            vertex_buffer, index_buffer, uniform_buffer,
        };
        Ok(target)
    }
}

impl vkbase::RenderWorkflow for VulkanExample {

    fn init(&mut self, device: &VkDevice) -> VkResult<()> {

        self.record_commands(device, self.backend.dimension)?;
        Ok(())
    }

    fn render_frame(&mut self, device: &mut VkDevice, device_available: vk::Fence, await_present: vk::Semaphore, image_index: usize, _delta_time: f32) -> VkResult<vk::Semaphore> {

        let submit_ci = vkbase::ci::device::SubmitCI::new()
            .add_wait(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, await_present)
            .add_command(self.backend.commands[image_index])
            .add_signal(self.backend.await_rendering);

        // Submit to the graphics queue passing a wait fence.
        device.submit(submit_ci, device.logic.queues.graphics.handle, device_available)?;

        Ok(self.backend.await_rendering)
    }

    fn swapchain_reload(&mut self, device: &mut VkDevice, new_chain: &VkSwapchain) -> VkResult<()> {

        // recreate the resources.
        device.discard(self.pipeline);

        let render_pass = setup_renderpass(device, new_chain)?;
        self.backend.swapchain_reload(device, new_chain, render_pass)?;
        self.pipeline = prepare_pipelines(device, self.backend.render_pass, self.descriptors.pipeline_layout)?;

        self.record_commands(device, self.backend.dimension)?;

        Ok(())
    }

    fn receive_input(&mut self, inputer: &vkbase::EventController, _delta_time: f32) -> FrameAction {

        if inputer.key.is_key_pressed(winit::VirtualKeyCode::Escape) {
            return FrameAction::Terminal
        }

        FrameAction::Rendering
    }

    fn deinit(self, device: &mut VkDevice) -> VkResult<()> {

        device.discard(self.descriptors.set_layout);
        device.discard(self.descriptors.descriptor_pool);

        device.discard(self.pipeline);
        device.discard(self.descriptors.pipeline_layout);

        device.discard(self.vertex_buffer.buffer);
        device.discard(self.vertex_buffer.memory);

        device.discard(self.index_buffer.buffer);
        device.discard(self.index_buffer.memory);

        device.discard(self.uniform_buffer.buffer);
        device.discard(self.uniform_buffer.memory);

        self.backend.discard_by(device)
    }
}

impl VulkanExample {

    fn record_commands(&self, device: &VkDevice, dimension: vk::Extent2D) -> VkResult<()> {

        let clear_values = vec![
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

        for (i, &command) in self.backend.commands.iter().enumerate() {

            use vkbase::command::{VkCmdRecorder, CmdGraphicsApi, IGraphics};
            use vkbase::ci::pipeline::RenderPassBI;

            let recorder: VkCmdRecorder<IGraphics> = VkCmdRecorder::new(&device.logic, command);

            let render_pass_bi = RenderPassBI::new(self.backend.render_pass, self.backend.framebuffers[i])
                .render_extent(dimension)
                .set_clear_values(clear_values.clone());

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
}

fn setup_descriptor(device: &VkDevice, uniforms: &UniformBuffer) -> VkResult<DescriptorStaff> {

    use vkbase::ci::descriptor::{DescriptorPoolCI, DescriptorSetLayoutCI};
    use vkbase::ci::descriptor::{DescriptorSetAI, DescriptorBufferSetWI, DescriptorSetsUpdateCI};
    use vkbase::ci::pipeline::PipelineLayoutCI;

    // Descriptor Pool.
    let descriptor_pool = DescriptorPoolCI::new(1)
        .add_descriptor(vk::DescriptorType::UNIFORM_BUFFER, 1)
        .build(device)?;

    // Descriptor Set Layout.
    let uniform_descriptor = vk::DescriptorSetLayoutBinding {
        binding: 0,
        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
        descriptor_count: 1,
        stage_flags: vk::ShaderStageFlags::VERTEX,
        p_immutable_samplers: ptr::null(),
    };

    let set_layout = DescriptorSetLayoutCI::new()
        .add_binding(uniform_descriptor)
        .build(device)?;

    // Pipeline Layout.
    let pipeline_layout = PipelineLayoutCI::new()
        .add_set_layout(set_layout)
        .build(device)?;

    // Descriptor set.
    let mut descriptor_sets = DescriptorSetAI::new(descriptor_pool)
        .add_set_layout(set_layout)
        .build(device)?;
    let descriptor_set = descriptor_sets.remove(0);

    let write_info = DescriptorBufferSetWI::new(descriptor_set, 0, vk::DescriptorType::UNIFORM_BUFFER)
        .add_buffer(uniforms.descriptor.clone());

    DescriptorSetsUpdateCI::new()
        .add_write(&write_info)
        .update(device);

    let result = DescriptorStaff { descriptor_pool, descriptor_set, pipeline_layout, set_layout };
    Ok(result)
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
        .add_attachment(color_attachment)
        .add_attachment(depth_attachment)
        .add_subpass(subpass_description)
        .add_dependency(dependency0)
        .add_dependency(dependency1)
        .build(device)?;

    Ok(render_pass)
}

fn prepare_pipelines(device: &VkDevice, render_pass: vk::RenderPass, layout: vk::PipelineLayout) -> VkResult<vk::Pipeline> {

    use vkbase::ci::pipeline::*;

    let viewport_state = ViewportSCI::new()
        .add_viewport(vk::Viewport::default())
        .add_scissor(vk::Rect2D::default());

    let blend_attachment = BlendAttachmentSCI::new();
    let blend_state = ColorBlendSCI::new()
        .add_attachment(blend_attachment);

    let depth_stencil_state = DepthStencilSCI::new()
        .depth_test(true, true, vk::CompareOp::LESS_OR_EQUAL);

    let dynamic_state = DynamicSCI::new()
        .add_dynamic(vk::DynamicState::VIEWPORT)
        .add_dynamic(vk::DynamicState::SCISSOR);

    let inputs = Vertex::input_description();
    let vertex_input_state = VertexInputSCI::new()
        .add_binding(inputs.binding)
        .add_attribute(inputs.attributes[0])
        .add_attribute(inputs.attributes[1]);

    // shaders
    use vkbase::ci::shader::{ShaderModuleCI, ShaderStageCI};

    let mut shader_compiler = vkbase::utils::shaderc::VkShaderCompiler::new()?;
    let vert_codes = shader_compiler.compile_from_path(Path::new(SHADER_VERTEX_PATH), shaderc::ShaderKind::Vertex, "[Vertex Shader]", "main")?;
    let frag_codes = shader_compiler.compile_from_path(Path::new(SHADER_FRAGMENT_PATH), shaderc::ShaderKind::Fragment, "[Fragment Shader]", "main")?;

    let vert_module = ShaderModuleCI::new(vert_codes)
        .build(device)?;
    let frag_module = ShaderModuleCI::new(frag_codes).build(device)?;


    let mut pipeline_ci = GraphicsPipelineCI::new(render_pass, layout);

    let shaders = [
        ShaderStageCI::new(vk::ShaderStageFlags::VERTEX, vert_module),
        ShaderStageCI::new(vk::ShaderStageFlags::FRAGMENT, frag_module),
    ];
    pipeline_ci.set_shaders(&shaders);
    pipeline_ci.set_vertex_input(vertex_input_state);
    pipeline_ci.set_viewport(viewport_state);
    pipeline_ci.set_depth_stencil(depth_stencil_state);
    pipeline_ci.set_color_blend(blend_state);
    pipeline_ci.set_dynamic(dynamic_state);

    let pipeline = device.build(&pipeline_ci)?;


    device.discard(vert_module);
    device.discard(frag_module);

    Ok(pipeline)
}

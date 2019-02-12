
use ash::vk;
use ash::version::DeviceV1_0;

use vkbase::context::{VkDevice, VkSwapchain};
use vkbase::ci::VkObjectBuildableCI;
use vkbase::{VkResult, VkError};
use vkbase::FrameAction;
use vkbase::vkuint;

use std::ptr;
use std::path::Path;

use crate::data::{Vertex, VertexBuffer, IndexBuffer, UniformBuffer, DepthImage};

const SHADER_VERTEX_PATH  : &'static str = "examples/src/triangle_v1/triangle.vert.glsl";
const SHADER_FRAGMENT_PATH: &'static str = "examples/src/triangle_v1/triangle.frag.glsl";

pub struct VulkanExample {

    dimension: vk::Extent2D,
    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,
    uniform_buffer: UniformBuffer,

    depth_image: DepthImage,

    render_pass: vk::RenderPass,

    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,

    framebuffers: Vec<vk::Framebuffer>,

    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_set: vk::DescriptorSet,

    command_pool: vk::CommandPool,
    commands: Vec<vk::CommandBuffer>,

    await_rendering: vk::Semaphore,
}

impl VulkanExample {

    pub fn new(context: &vkbase::context::VulkanContext) -> VkResult<VulkanExample> {

        let device = &context.device;
        let swapchain = &context.swapchain;
        let dimension = swapchain.dimension;

        let command_pool = super::helper::create_command_pool(device)?;
        let commands = create_command_buffer(device, command_pool, swapchain.frame_in_flight as _)?;

        let (vertex_buffer, index_buffer) = super::data::prepare_vertices(device, command_pool)?;
        let uniform_buffer = super::data::prepare_uniform(device, dimension)?;

        let descriptor_pool = setup_descriptor_pool(device)?;
        let (descriptor_set_layout, pipeline_layout) = setup_descriptor_layout(device)?;
        let descriptor_set = setup_descriptor_set(device, descriptor_pool, descriptor_set_layout, &uniform_buffer)?;

        let render_pass = setup_renderpass(device, &context.swapchain)?;
        let depth_image = setup_depth_stencil(device, dimension)?;
        let framebuffers = setup_framebuffers(device, &context.swapchain, render_pass, &depth_image)?;
        let pipeline = prepare_pipelines(device, render_pass, pipeline_layout)?;

        let await_rendering = setup_sync_primitive(device)?;

        let target = VulkanExample {
            command_pool, commands,
            descriptor_pool, descriptor_set, descriptor_set_layout,
            pipeline, pipeline_layout, render_pass, framebuffers,
            vertex_buffer, index_buffer, uniform_buffer, depth_image, dimension,
            await_rendering,
        };
        Ok(target)
    }
}

impl vkbase::Workflow for VulkanExample {

    fn init(&mut self, device: &VkDevice) -> VkResult<()> {

        self.record_commands(device, self.dimension)?;
        Ok(())
    }

    fn render_frame(&mut self, device: &VkDevice, device_available: vk::Fence, await_present: vk::Semaphore, image_index: usize, _delta_time: f32) -> VkResult<vk::Semaphore> {

        let submit_infos = [
            vk::SubmitInfo {
                s_type: vk::StructureType::SUBMIT_INFO,
                p_next: ptr::null(),
                wait_semaphore_count   : 1,
                p_wait_semaphores      : &await_present,
                // Pipeline stage at which the queue submission will wait (via p_wait_semaphores).
                p_wait_dst_stage_mask  : &vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                command_buffer_count   : 1,
                p_command_buffers      : &self.commands[image_index],
                signal_semaphore_count : 1,
                p_signal_semaphores    : &self.await_rendering,
            },
        ];

        // Submit to the graphics queue passing a wait fence.
        unsafe {
            device.logic.handle.queue_submit(device.logic.queues.graphics.handle, &submit_infos, device_available)
                .map_err(|_| VkError::device("Queue Submit"))?;
        }

        Ok(self.await_rendering)
    }

    fn swapchain_reload(&mut self, device: &VkDevice, new_chain: &VkSwapchain) -> VkResult<()> {

        unsafe {
            device.logic.handle.destroy_pipeline(self.pipeline, None);
        }

        device.discard(self.render_pass);
        device.discard(&self.framebuffers);

        device.discard(self.depth_image.view);
        device.discard(self.depth_image.image);
        device.discard(self.depth_image.memory);

        // recreate the resources.
        unsafe {

            self.dimension = new_chain.dimension;
            self.render_pass = setup_renderpass(device, new_chain)?;
            self.depth_image = setup_depth_stencil(device, self.dimension)?;

            self.framebuffers = setup_framebuffers(device, new_chain, self.render_pass, &self.depth_image)?;
            self.pipeline = prepare_pipelines(device, self.render_pass, self.pipeline_layout)?;

            device.logic.handle.reset_command_pool(self.command_pool, vk::CommandPoolResetFlags::RELEASE_RESOURCES)
                .map_err(|_| VkError::device("Reset Command Poll"))?;
            self.record_commands(device, self.dimension)?;
        }

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

        for (i, &command) in self.commands.iter().enumerate() {

            use vkbase::command::{VkCmdRecorder, CmdGraphicsApi, IGraphics};
            use vkbase::ci::pipeline::RenderPassBI;

            let recorder: VkCmdRecorder<IGraphics> = VkCmdRecorder::new(device, command);

            let render_pass_bi = RenderPassBI::new(self.render_pass, self.framebuffers[i])
                .render_extent(dimension)
                .clear_values(&clear_values);

            recorder.begin_record()?
                .begin_render_pass(render_pass_bi)
                .set_viewport(0, &[viewport])
                .set_scissor(0, &[scissor])
                .bind_descriptor_sets(self.pipeline_layout, 0, &[self.descriptor_set], &[])
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

        device.discard(self.descriptor_set_layout);
        device.discard(self.descriptor_pool);

        unsafe {
            device.logic.handle.destroy_pipeline(self.pipeline, None);
        }

        device.discard(self.pipeline_layout);
        device.discard(self.render_pass);
        device.discard(&self.framebuffers);

        device.discard(self.command_pool);

        device.discard(self.depth_image.view);
        device.discard(self.depth_image.image);
        device.discard(self.depth_image.memory);

        device.discard(self.vertex_buffer.buffer);
        device.discard(self.vertex_buffer.memory);

        device.discard(self.index_buffer.buffer);
        device.discard(self.index_buffer.memory);

        device.discard(self.uniform_buffer.buffer);
        device.discard(self.uniform_buffer.memory);

        device.discard(self.await_rendering);
    }
}


pub fn create_command_buffer(device: &VkDevice, pool: vk::CommandPool, count: vkuint) -> VkResult<Vec<vk::CommandBuffer>> {

    use vkbase::ci::command::CommandBufferAI;
    let command_buffers = CommandBufferAI::new(pool, count)
        .build(device)?;
    Ok(command_buffers)
}

fn setup_descriptor_pool(device: &VkDevice) -> VkResult<vk::DescriptorPool> {

    use vkbase::ci::descriptor::DescriptorPoolCI;

    let descriptor_pool = DescriptorPoolCI::new(1)
        .add_descriptor(vk::DescriptorType::UNIFORM_BUFFER, 1)
        .build(device)?;
    Ok(descriptor_pool)
}

fn setup_descriptor_layout(device: &VkDevice) -> VkResult<(vk::DescriptorSetLayout, vk::PipelineLayout)> {

    use vkbase::ci::descriptor::DescriptorSetLayoutCI;
    use vkbase::ci::pipeline::PipelineLayoutCI;

    // Descriptor Set Layout.
    let uniform_descriptor = vk::DescriptorSetLayoutBinding {
        binding: 0,
        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
        descriptor_count: 1,
        stage_flags: vk::ShaderStageFlags::VERTEX,
        p_immutable_samplers: ptr::null(),
    };

    let descriptor_set_layout = DescriptorSetLayoutCI::new()
        .add_binding(uniform_descriptor)
        .build(device)?;

    // Pipeline Layout.
    let pipeline_layout = PipelineLayoutCI::new()
        .add_set_layout(descriptor_set_layout)
        .build(device)?;

    Ok((descriptor_set_layout, pipeline_layout))
}

fn setup_descriptor_set(device: &VkDevice, pool: vk::DescriptorPool, set_layout: vk::DescriptorSetLayout, uniforms: &UniformBuffer) -> VkResult<vk::DescriptorSet> {

    use vkbase::ci::descriptor::{DescriptorSetAI, DescriptorBufferSetWI};

    let descriptor_set = DescriptorSetAI::new(pool)
        .add_set_layout(set_layout)
        .build(device)?
        .remove(0);

    let write_info = DescriptorBufferSetWI::new(descriptor_set, 0, vk::DescriptorType::UNIFORM_BUFFER)
        .add_buffer(uniforms.descriptor.clone());

    unsafe {
        device.logic.handle.update_descriptor_sets(&[write_info.value()], &[]);
    }

    Ok(descriptor_set)
}


fn setup_depth_stencil(device: &VkDevice, dimension: vk::Extent2D) -> VkResult<DepthImage> {

    use vkbase::ci::image::{ImageCI, ImageViewCI};
    use vkbase::ci::memory::MemoryAI;

    let (image, image_requirement) = ImageCI::new_2d(device.phy.depth_format, dimension)
        .usages(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
        .build(device)?;

    let memory_index = super::helper::get_memory_type_index(device, image_requirement.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL);
    let memory = MemoryAI::new(image_requirement.size, memory_index)
        .build(device)?;

    unsafe {
        device.logic.handle.bind_image_memory(image, memory, 0)
            .map_err(|_| VkError::device("Bind Image Memory."))?;
    }

    let view = ImageViewCI::new(image, vk::ImageViewType::TYPE_2D, device.phy.depth_format)
        .aspect_mask(vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL)
        .build(device)?;

    let result = DepthImage { image, view, memory };
    Ok(result)
}

fn setup_renderpass(device: &VkDevice, swapchain: &VkSwapchain) -> VkResult<vk::RenderPass> {

    use vkbase::ci::pipeline::RenderPassCI;

    let color_attachment = vk::AttachmentDescription {
        flags: vk::AttachmentDescriptionFlags::empty(),
        format: swapchain.backend_format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::STORE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
    };

    let depth_attachment = vk::AttachmentDescription {
        flags: vk::AttachmentDescriptionFlags::empty(),
        format: device.phy.depth_format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::DONT_CARE,
        stencil_load_op  : vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op : vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    };


    let color_refs = [
        vk::AttachmentReference {
            attachment: 0, // Attachment 0 is color.
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }
    ];
    let depth_ref = vk::AttachmentReference {
        attachment: 1, // Attachment 0 is depth-stencil.
        layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    };

    let subpass_description = vk::SubpassDescription {
        flags                      : vk::SubpassDescriptionFlags::empty(),
        pipeline_bind_point        : vk::PipelineBindPoint::GRAPHICS,
        input_attachment_count     : 0,
        // Input attachments can be used to sample from contents of a previous subpass.
        p_input_attachments        : ptr::null(),
        // Reference to the color attachment in slot 0.
        color_attachment_count     : color_refs.len() as _,
        p_color_attachments        : color_refs.as_ptr(),
        // Resolve attachments are resolved at the end of a sub pass and can be used for e.g. multi sampling.
        p_resolve_attachments      : ptr::null(),
        // Reference to the depth attachment in slot 1.
        p_depth_stencil_attachment : &depth_ref,
        // Preserved attachments can be used to loop (and preserve) attachments through subpasses.
        preserve_attachment_count  : 0,
        p_preserve_attachments     : ptr::null(),
    };

    let dependencies = [
        // First dependency at the start of the renderpass does the transition from final to initial layout.
        vk::SubpassDependency {
            // Producer of the dependency.
            src_subpass: vk::SUBPASS_EXTERNAL,
            // Consumer is our single subpass that will wait for the execution dependency.
            dst_subpass: 0,
            src_stage_mask   : vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            dst_stage_mask   : vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask  : vk::AccessFlags::MEMORY_READ,
            dst_access_mask  : vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dependency_flags : vk::DependencyFlags::BY_REGION,
        },
        // Second dependency at the end the renderpass does the transition from the initial to the final layout.
        vk::SubpassDependency {
            // Producer of the dependency is our single subpass.
            src_subpass: 0,
            // Consumer are all commands outside of the renderpass.
            dst_subpass: vk::SUBPASS_EXTERNAL,
            src_stage_mask   : vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_stage_mask   : vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            src_access_mask  : vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dst_access_mask  : vk::AccessFlags::MEMORY_READ,
            dependency_flags : vk::DependencyFlags::BY_REGION,
        },
    ];

    let render_pass = RenderPassCI::new()
        .add_attachment(color_attachment)
        .add_attachment(depth_attachment)
        .add_subpass(subpass_description)
        .add_dependency(dependencies[0])
        .add_dependency(dependencies[1])
        .build(device)?;

    Ok(render_pass)
}

// Create a frame buffer for each swap chain image.
fn setup_framebuffers(device: &VkDevice, swapchain: &VkSwapchain, render_pass: vk::RenderPass, depth_image: &DepthImage) -> VkResult<Vec<vk::Framebuffer>> {

    use vkbase::ci::pipeline::FramebufferCI;

    // create a frame buffer for every image in the swapchain.
    let mut framebuffers = Vec::with_capacity(swapchain.frame_in_flight());
    let dimension = swapchain.dimension.clone();

    for i in 0..swapchain.frame_in_flight() {

        let framebuffer = FramebufferCI::new_2d(render_pass, dimension)
            .add_attachment(swapchain.images[i].view) // color attachment is the view of the swapchain image.
            .add_attachment(depth_image.view) // depth/stencil attachment is the same for all frame buffers.
            .build(device)?;
        framebuffers.push(framebuffer);
    }

    Ok(framebuffers)
}


fn prepare_pipelines(device: &VkDevice, render_pass: vk::RenderPass, layout: vk::PipelineLayout) -> VkResult<vk::Pipeline> {

    use vkbase::ci::pipeline::*;

    let input_assembly_state = InputAssemblySCI::new();

    let rasterization_state = RasterizationSCI::new();

    let viewport_state = ViewportSCI::new()
        .add_viewport(vk::Viewport::default())
        .add_scissor(vk::Rect2D::default());

    let blend_attachment = BlendAttachmentSCI::new().value();
    let blend_state = ColorBlendSCI::new()
        .add_attachment(blend_attachment);

    let depth_stencil_state = DepthStencilSCI::new()
        .depth_test(true, true, vk::CompareOp::LESS_OR_EQUAL);

    let multisample_state = MultisampleSCI::new();

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

    let vert_module = ShaderModuleCI::from_glsl(vk::ShaderStageFlags::VERTEX, vert_codes)
        .build(device)?;
    let frag_module = ShaderModuleCI::from_glsl(vk::ShaderStageFlags::FRAGMENT, frag_codes)
        .build(device)?;

    let vert_sci = ShaderStageCI::new(vk::ShaderStageFlags::VERTEX, vert_module);
    let frag_sci = ShaderStageCI::new(vk::ShaderStageFlags::FRAGMENT, frag_module);

    let mut pipeline_ci = GraphicsPipelineCI::new(render_pass, layout);

    pipeline_ci.add_shader_stage(vert_sci);
    pipeline_ci.add_shader_stage(frag_sci);

    pipeline_ci.set_vertex_input(vertex_input_state);
    pipeline_ci.set_input_assembly(input_assembly_state);
    pipeline_ci.set_viewport(viewport_state);
    pipeline_ci.set_rasterization(rasterization_state);
    pipeline_ci.set_multisample(multisample_state);
    pipeline_ci.set_depth_stencil(depth_stencil_state);
    pipeline_ci.set_color_blend(blend_state);
    pipeline_ci.set_dynamic(dynamic_state);

    let pipeline = device.build(pipeline_ci)?;


    device.discard(vert_module);
    device.discard(frag_module);

    Ok(pipeline)
}

fn setup_sync_primitive(device: &VkDevice) -> VkResult<vk::Semaphore> {

    use vkbase::ci::sync::SemaphoreCI;
    let semaphore = SemaphoreCI::new().build(device)?;
    Ok(semaphore)
}


use ash::vk;

use std::ptr;
use std::path::Path;

use vkbase::context::{VulkanContext, VkDevice, VkSwapchain};
use vkbase::ci::VkObjectBuildableCI;
use vkbase::ci::vma::VmaBuffer;
use vkbase::{FlightCamera, FrameAction};
use vkbase::{vkbytes, vkuint, vkptr, Point3F};
use vkbase::{VkResult, VkErrorKind};

use vkexamples::VkExampleBackendRes;
use crate::data::{OBJECT_INSTANCES, INDEX_DATA, Vertex, RotationData, UboViewData, UboDynamicData};

const SHADER_VERTEX_PATH  : &'static str = "examples/src/dynamicuniformbuffer/base.vert.glsl";
const SHADER_FRAGMENT_PATH: &'static str = "examples/src/dynamicuniformbuffer/base.frag.glsl";

pub struct VulkanExample {

    backend: VkExampleBackendRes,
    camera: FlightCamera,
    time_counter: f32,

    vertices: VmaBuffer,
    indices : VmaBuffer,

    ubo_view: VmaBuffer,
    ubo_view_data: UboViewData,

    ubo_dynamics: VmaBuffer,
    ubo_dynamics_data: UboDynamicData,
    rotations: RotationData,
    dynamic_alignment: vkuint,

    pipelines: PipelineStaff,
    descriptors: DescriptorStaff,

    is_toggle_event: bool,
}

impl VulkanExample {

    pub fn new(context: &mut VulkanContext) -> VkResult<VulkanExample> {

        let device = &mut context.device;
        let swapchain = &context.swapchain;
        let dimension = swapchain.dimension;

        let mut camera = FlightCamera::new()
            .place_at(Point3F::new(0.0, 0.0, 34.0))
            .screen_aspect_ratio(dimension.width as f32 / dimension.height as f32)
            .build();
        camera.set_move_speed(50.0);

        let render_pass = setup_renderpass(device, &context.swapchain)?;
        let backend = VkExampleBackendRes::new(device, swapchain, render_pass)?;

        let (vertices, indices) = super::data::generate_cube(device)?;
        let (ubo_view, ubo_view_data) = UboViewData::prepare_buffer(device, &camera)?;
        let (ubo_dynamics, ubo_dynamics_data, dynamic_alignment) = UboDynamicData::prepare_buffer(device)?;
        let rotations = RotationData::new_by_rng();

        let descriptors = setup_descriptor(device, &ubo_view, &ubo_dynamics, dynamic_alignment)?;

        let pipelines = prepare_pipelines(device, backend.render_pass, descriptors.layout)?;

        let target = VulkanExample {
            backend, descriptors, pipelines, camera,
            vertices, indices,
            ubo_view, ubo_view_data, rotations,
            ubo_dynamics, ubo_dynamics_data, dynamic_alignment,
            time_counter: 0.0,
            is_toggle_event: false,
        };
        Ok(target)
    }
}

impl vkbase::RenderWorkflow for VulkanExample {

    fn init(&mut self, device: &VkDevice) -> VkResult<()> {

        self.backend.set_basic_ui(device, super::WINDOW_TITLE)?;

        self.record_commands(device, self.backend.dimension)?;
        Ok(())
    }

    fn render_frame(&mut self, device: &mut VkDevice, device_available: vk::Fence, await_present: vk::Semaphore, image_index: usize, delta_time: f32) -> VkResult<vk::Semaphore> {

        self.update_uniforms(device, delta_time)?;

        let submit_ci = vkbase::ci::device::SubmitCI::new()
            .add_wait(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, await_present)
            .add_command(self.backend.commands[image_index])
            .add_signal(self.backend.await_rendering);

        device.submit(submit_ci, device.logic.queues.graphics.handle, device_available)?;

        Ok(self.backend.await_rendering)
    }

    fn swapchain_reload(&mut self, device: &mut VkDevice, new_chain: &VkSwapchain) -> VkResult<()> {

        // recreate the resources.
        device.discard(self.pipelines.pipeline);

        let render_pass = setup_renderpass(device, new_chain)?;
        self.backend.swapchain_reload(device, new_chain, render_pass)?;
        self.pipelines = prepare_pipelines(device, self.backend.render_pass, self.descriptors.layout)?;

        self.record_commands(device, self.backend.dimension)?;

        Ok(())
    }

    fn receive_input(&mut self, inputer: &vkbase::EventController, delta_time: f32) -> FrameAction {

        if inputer.is_key_active() || inputer.is_cursor_active() {

            if inputer.key.is_key_pressed(winit::VirtualKeyCode::Escape) {
                return FrameAction::Terminal
            }

            self.is_toggle_event = true;
            self.camera.receive_input(inputer, delta_time);
        } else {
            self.is_toggle_event = false;
        }

        self.backend.update_fps_text(inputer);

        FrameAction::Rendering
    }

    fn deinit(self, device: &mut VkDevice) -> VkResult<()> {

        self.discard(device)
    }
}

impl VulkanExample {

    fn record_commands(&self, device: &VkDevice, dimension: vk::Extent2D) -> VkResult<()> {

        let clear_values = [
            vkexamples::DEFAULT_CLEAR_COLOR.clone(),
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
                .clear_values(&clear_values);

            recorder.begin_record()?
                .begin_render_pass(render_pass_bi)
                .set_viewport(0, &[viewport])
                .set_scissor(0, &[scissor])
                .bind_pipeline(self.pipelines.pipeline)
                .bind_vertex_buffers(0, &[self.vertices.handle], &[0])
                .bind_index_buffer(self.indices.handle, vk::IndexType::UINT32, 0);

            // Render multiple objects using different model matrices by dynamically offsetting into one uniform buffer.
            for i in 0..(OBJECT_INSTANCES as vkuint) {
                // One dynamic offset per dynamic descriptor to offset into the ubo containing all model matrices.
                let dynamic_offset = i * self.dynamic_alignment;
                recorder
                    .bind_descriptor_sets(self.pipelines.layout, 0, &[self.descriptors.set], &[dynamic_offset])
                    .draw_indexed(INDEX_DATA.len() as vkuint, 1, 0, 0, 0);
            }

            self.backend.ui_renderer.record_command(&recorder);

            recorder.end_render_pass()
                .end_record()?;
        }

        Ok(())
    }

    fn update_uniforms(&mut self, device: &mut VkDevice, delta_time: f32) -> VkResult<()> {

        const FPS_60: f32 = 1.0 / 60.0;

        // update 60 times per second.
        self.time_counter += delta_time;
        if self.time_counter > FPS_60 {

            self.time_counter = 0.0;

            { // update camera.
                self.ubo_view_data.content[0].view = self.camera.view_matrix();
                let data_ptr = self.ubo_view.info.get_mapped_data() as vkptr;
                vkbase::utils::memory::copy_to_ptr(data_ptr, &self.ubo_view_data.content);
            }

            { // update models.
                self.ubo_dynamics_data.update(&mut self.rotations, delta_time);
                let data_ptr = self.ubo_dynamics.info.get_mapped_data() as vkptr;

                let mut data_ptr_aligned = unsafe {
                    ash::util::Align::new(data_ptr, self.dynamic_alignment as _, self.ubo_dynamics.info.get_size() as _)
                };
                data_ptr_aligned.copy_from_slice(&self.ubo_dynamics_data.model);

                device.vma.flush_allocation(&self.ubo_dynamics.allocation, 0, vk::WHOLE_SIZE as _)
                    .map_err(VkErrorKind::Vma)?;
            }
        }

        Ok(())
    }

    fn discard(self, device: &mut VkDevice) -> VkResult<()> {

        device.discard(self.descriptors.layout);
        device.discard(self.descriptors.pool);

        device.discard(self.pipelines.pipeline);
        device.discard(self.pipelines.layout);

        device.vma_discard(self.vertices)?;
        device.vma_discard(self.indices)?;
        device.vma_discard(self.ubo_view)?;
        device.vma_discard(self.ubo_dynamics)?;

        self.backend.discard_by(device)
    }
}



struct DescriptorStaff {
    pool   : vk::DescriptorPool,
    set    : vk::DescriptorSet,
    layout : vk::DescriptorSetLayout,
}

fn setup_descriptor(device: &VkDevice, ubo_view: &VmaBuffer, ubo_dynamics: &VmaBuffer, dynamic_alignment: vkuint) -> VkResult<DescriptorStaff> {

    use vkbase::ci::descriptor::{DescriptorPoolCI, DescriptorSetLayoutCI};
    use vkbase::ci::descriptor::{DescriptorSetAI, DescriptorBufferSetWI, DescriptorSetsUpdateCI};

    // Descriptor Pool.
    let descriptor_pool = DescriptorPoolCI::new(1)
        .add_descriptor(vk::DescriptorType::UNIFORM_BUFFER, 1)
        .add_descriptor(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC, 1)
        .build(device)?;

    // in base.vert.glsl:
    // layout (set = 0, binding = 0) uniform UboView {
    //     mat4 projection;
    //     mat4 view;
    // } uboView;
    let ubo_view_descriptor = vk::DescriptorSetLayoutBinding {
        binding: 0,
        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
        descriptor_count: 1,
        stage_flags: vk::ShaderStageFlags::VERTEX,
        p_immutable_samplers: ptr::null(),
    };

    // in base.vert.glsl:
    // layout (set = 0, binding = 1) uniform UboInstance {
    //     mat4 model;
    // } uboInstance;
    let ubo_dynamics_descriptor = vk::DescriptorSetLayoutBinding {
        binding: 1,
        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
        descriptor_count: 1,
        stage_flags: vk::ShaderStageFlags::VERTEX,
        p_immutable_samplers: ptr::null(),
    };

    let set_layout = DescriptorSetLayoutCI::new()
        .add_binding(ubo_view_descriptor)
        .add_binding(ubo_dynamics_descriptor)
        .build(device)?;

    // Descriptor set.
    let mut descriptor_sets = DescriptorSetAI::new(descriptor_pool)
        .add_set_layout(set_layout)
        .build(device)?;
    let descriptor_set = descriptor_sets.remove(0);

    let ubo_view_write = DescriptorBufferSetWI::new(descriptor_set, 0, vk::DescriptorType::UNIFORM_BUFFER)
        .add_buffer(vk::DescriptorBufferInfo {
            buffer: ubo_view.handle,
            offset: 0,
            range : ubo_view.info.get_size() as _,
        });

    let ubo_dynamic_write = DescriptorBufferSetWI::new(descriptor_set, 1, vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
        .add_buffer(vk::DescriptorBufferInfo {
            buffer: ubo_dynamics.handle,
            offset: 0,
            range : dynamic_alignment as vkbytes,
        });

    DescriptorSetsUpdateCI::new()
        .add_write(ubo_view_write.value())
        .add_write(ubo_dynamic_write.value())
        .update(device);

    let result = DescriptorStaff {
        pool: descriptor_pool,
        set : descriptor_set,
        layout: set_layout,
    };
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
        .add_attachment(color_attachment.value())
        .add_attachment(depth_attachment.value())
        .add_subpass(subpass_description.value())
        .add_dependency(dependency0.value())
        .add_dependency(dependency1.value())
        .build(device)?;

    Ok(render_pass)
}


struct PipelineStaff {
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
}

fn prepare_pipelines(device: &VkDevice, render_pass: vk::RenderPass, set_layout: vk::DescriptorSetLayout) -> VkResult<PipelineStaff> {

    use vkbase::ci::pipeline::*;

    let viewport_state = ViewportSCI::new()
        .add_viewport(vk::Viewport::default())
        .add_scissor(vk::Rect2D::default());

    let rasterization_state = RasterizationSCI::new()
        .polygon(vk::PolygonMode::FILL)
        .cull_face(vk::CullModeFlags::BACK, vk::FrontFace::CLOCKWISE);

    let blend_attachment = BlendAttachmentSCI::new().value();
    let blend_state = ColorBlendSCI::new()
        .add_attachment(blend_attachment);

    let depth_stencil_state = DepthStencilSCI::new()
        .depth_test(true, true, vk::CompareOp::LESS_OR_EQUAL);

    let dynamic_state = DynamicSCI::new()
        .add_dynamic(vk::DynamicState::VIEWPORT)
        .add_dynamic(vk::DynamicState::SCISSOR);

    // shaders
    use vkbase::ci::shader::{ShaderModuleCI, ShaderStageCI};

    let mut shader_compiler = vkbase::utils::shaderc::VkShaderCompiler::new()?;
    let vert_codes = shader_compiler.compile_from_path(Path::new(SHADER_VERTEX_PATH), shaderc::ShaderKind::Vertex, "[Vertex Shader]", "main")?;
    let frag_codes = shader_compiler.compile_from_path(Path::new(SHADER_FRAGMENT_PATH), shaderc::ShaderKind::Fragment, "[Fragment Shader]", "main")?;

    let vert_module = ShaderModuleCI::from_glsl(vk::ShaderStageFlags::VERTEX, vert_codes)
        .build(device)?;
    let frag_module = ShaderModuleCI::from_glsl(vk::ShaderStageFlags::FRAGMENT, frag_codes)
        .build(device)?;

    // Pipeline Layout.
    let layout = PipelineLayoutCI::new()
        .add_set_layout(set_layout)
        .build(device)?;

    let mut pipeline_ci = GraphicsPipelineCI::new(render_pass, layout);

    pipeline_ci.set_shaders(vec![
        ShaderStageCI::new(vk::ShaderStageFlags::VERTEX, vert_module),
        ShaderStageCI::new(vk::ShaderStageFlags::FRAGMENT, frag_module),
    ]);
    pipeline_ci.set_vertex_input(Vertex::input_description());
    pipeline_ci.set_viewport(viewport_state);
    pipeline_ci.set_depth_stencil(depth_stencil_state);
    pipeline_ci.set_rasterization(rasterization_state);
    pipeline_ci.set_color_blend(blend_state);
    pipeline_ci.set_dynamic(dynamic_state);

    let pipeline = device.build(&pipeline_ci)?;

    device.discard(vert_module);
    device.discard(frag_module);

    let result = PipelineStaff { pipeline, layout };
    Ok(result)
}

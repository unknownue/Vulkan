
use ash::vk;

use std::ptr;
use std::mem;
use std::path::Path;

use vkbase::context::{VkDevice, VkSwapchain};
use vkbase::ci::VkObjectBuildableCI;
use vkbase::context::VulkanContext;
use vkbase::{FlightCamera, FrameAction};
use vkbase::{vkbytes, vkptr, Point3F, Vector3F, Matrix4F};
use vkbase::VkResult;

use vkexamples::VkExampleBackend;
use super::data::{Skybox, UBOVS};

const SKY_BOX_VERTEX_SHADER_SOURCE_PATH  : &'static str = "examples/src/texturecubemap/skybox.vert.glsl";
const SKY_BOX_FRAGMENT_SHADER_SOURCE_PATH: &'static str = "examples/src/texturecubemap/skybox.frag.glsl";

/// Check box to toggle skybox display is not implement yet.
const DISPLAY_SKYBOX: bool = true;


pub struct VulkanExample {

    backend: VkExampleBackend,

    skybox: Skybox,

    pipelines: PipelineStaff,
    descriptors: DescriptorStaff,

    camera: FlightCamera,

    is_toggle_event: bool,
}

impl VulkanExample {

    pub fn new(context: &mut VulkanContext) -> VkResult<VulkanExample> {

        let device = &mut context.device;
        let swapchain = &context.swapchain;
        let dimension = swapchain.dimension;

        let mut camera = FlightCamera::new()
            .place_at(Point3F::new(0.0, 0.0, 0.0))
            .screen_aspect_ratio(dimension.width as f32 / dimension.height as f32)
            .build();
        camera.set_move_speed(10.0);


        let render_pass = setup_renderpass(device, &context.swapchain)?;
        let backend = VkExampleBackend::new(device, swapchain, render_pass)?;

        let mut skybox = Skybox::load_meshes(device, &camera)?;

        let descriptors = setup_descriptor(device, &mut skybox)?;

        let pipelines = prepare_pipelines(device, &skybox, backend.render_pass, descriptors.layout)?;

        let target = VulkanExample {
            backend, skybox, descriptors, pipelines, camera,
            is_toggle_event: true,
        };
        Ok(target)
    }
}

impl vkbase::RenderWorkflow for VulkanExample {

    fn init(&mut self, device: &VkDevice) -> VkResult<()> {

        self.backend.set_basic_ui(device, super::WINDOW_TITLE)?;

        self.update_uniforms(0.0)?;
        self.record_commands(device, self.backend.dimension)?;

        Ok(())
    }

    fn render_frame(&mut self, device: &mut VkDevice, device_available: vk::Fence, await_present: vk::Semaphore, image_index: usize, delta_time: f32) -> VkResult<vk::Semaphore> {

        self.update_uniforms(delta_time)?;

        let submit_ci = vkbase::ci::device::SubmitCI::new()
            .add_wait(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, await_present)
            .add_command(self.backend.commands[image_index])
            .add_signal(self.backend.await_rendering);

        device.submit(submit_ci, device.logic.queues.graphics.handle, device_available)?;

        Ok(self.backend.await_rendering)
    }

    fn swapchain_reload(&mut self, device: &mut VkDevice, new_chain: &VkSwapchain) -> VkResult<()> {

        device.discard(self.pipelines.skybox);

        let render_pass = setup_renderpass(device, new_chain)?;
        self.backend.swapchain_reload(device, new_chain, render_pass)?;
        self.pipelines = prepare_pipelines(device, &self.skybox, self.backend.render_pass, self.descriptors.layout)?;

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

        device.discard(self.descriptors.layout);
        device.discard(self.descriptors.pool);

        device.discard(self.pipelines.skybox);
        device.discard(self.pipelines.layout);

        self.skybox.discard_by(device)?;
        self.backend.discard_by(device)
    }
}

impl VulkanExample {

    fn record_commands(&self, device: &VkDevice, dimension: vk::Extent2D) -> VkResult<()> {

        let clear_values = [
            vkexamples::DEFAULT_CLEAR_COLOR.clone(),
            vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 } },
        ];

        let scissor = vk::Rect2D {
            extent: dimension.clone(),
            offset: vk::Offset2D { x: 0, y: 0 },
        };

        for (i, &command) in self.backend.commands.iter().enumerate() {

            use vkbase::command::{VkCmdRecorder, CmdGraphicsApi, IGraphics};
            use vkbase::ci::pipeline::RenderPassBI;

            let viewport = vk::Viewport {
                x: 0.0, y: 0.0,
                width: dimension.width as f32, height: dimension.height as f32,
                min_depth: 0.0, max_depth: 1.0,
            };

            let recorder: VkCmdRecorder<IGraphics> = VkCmdRecorder::new(&device.logic, command);

            let render_pass_bi = RenderPassBI::new(self.backend.render_pass, self.backend.framebuffers[i])
                .render_extent(dimension)
                .clear_values(&clear_values);

            recorder.begin_record()?
                .begin_render_pass(render_pass_bi)
                .set_viewport(0, &[viewport])
                .set_scissor(0, &[scissor]);

            if DISPLAY_SKYBOX { // render skybox

                recorder.bind_pipeline(self.pipelines.skybox);

                let render_params = vkbase::gltf::ModelRenderParams {
                    descriptor_set : self.skybox.descriptor_set,
                    pipeline_layout: self.pipelines.layout,
                    material_stage : None,
                };

                self.skybox.model.record_command(&recorder, &render_params);
            }

            self.backend.ui_renderer.record_command(&recorder);

            recorder
                .end_render_pass()
                .end_record()?;
        }

        Ok(())
    }

    fn update_uniforms(&mut self, _delta_time: f32) -> VkResult<()> {

        if self.is_toggle_event {

            let camera_pos = self.camera.current_position();
            let skybox_translation = Vector3F::new(camera_pos.x, camera_pos.y, camera_pos.z);

            // Magic number to adjust rotation.
            let camera_rotation = Matrix4F::new_rotation(Vector3F::new(-1.41, -0.8, -0.82));

            self.skybox.ubo_data[0].model = self.camera.view_matrix() * Matrix4F::new_translation(&skybox_translation) * camera_rotation;
            //self.skybox.ubo_data[0].model = self.camera.view_matrix();

            use vkbase::utils::memory::copy_to_ptr;
            copy_to_ptr(self.skybox.ubo_buffer.info.get_mapped_data() as vkptr, &self.skybox.ubo_data);
        }

        Ok(())
    }
}


struct DescriptorStaff {
    pool   : vk::DescriptorPool,
    layout : vk::DescriptorSetLayout,
}

fn setup_descriptor(device: &VkDevice, skybox: &mut Skybox) -> VkResult<DescriptorStaff> {

    use vkbase::ci::descriptor::{DescriptorPoolCI, DescriptorSetLayoutCI};
    use vkbase::ci::descriptor::{DescriptorSetAI, DescriptorBufferSetWI, DescriptorImageSetWI, DescriptorSetsUpdateCI};

    let descriptor_pool = DescriptorPoolCI::new(2)
        .add_descriptor(vk::DescriptorType::UNIFORM_BUFFER, 2)
        .add_descriptor(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC, 2)
        .add_descriptor(vk::DescriptorType::COMBINED_IMAGE_SAMPLER, 2)
        .build(device)?;

    /*
        Binding 0: Uniform buffer(share the same uniform buffer between skybox.vert.glsl and reflect.vert.glsl).
        in reflect.vert.glsl:

        layout (set = 0, binding = 0) uniform UBO {
           mat4 projection;
           mat4 model;
           float lodBias;
        } ubo;
    */
    let ubo_descriptor = vk::DescriptorSetLayoutBinding {
        binding: 0,
        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
        descriptor_count: 1,
        stage_flags: vk::ShaderStageFlags::VERTEX,
        p_immutable_samplers: ptr::null(),
    };

    /*
        Binding 1: Dynamic uniform buffer(used for matrix properties in glTF Node hierarchy).
        in skybox.vert.glsl:

        layout (set = 0, binding = 1) uniform DynNode {
            mat4 transform;
        } dyn_node;
    */
    let node_descriptor = vk::DescriptorSetLayoutBinding {
        binding: 1,
        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
        descriptor_count: 1,
        stage_flags: vk::ShaderStageFlags::VERTEX,
        p_immutable_samplers: ptr::null(),
    };

    /*
        Binding 2: Combined Image sampler.
        in skybox.frag.glsl or reflect.frag.glsl:

        layout (set = 0, binding = 2) uniform samplerCube samplerCubeMap;
    */
    let sampler_descriptor = vk::DescriptorSetLayoutBinding {
        binding: 2,
        descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        descriptor_count: 1,
        stage_flags: vk::ShaderStageFlags::FRAGMENT,
        p_immutable_samplers: ptr::null(),
    };

    let set_layout = DescriptorSetLayoutCI::new()
        .add_binding(ubo_descriptor)
        .add_binding(node_descriptor)
        .add_binding(sampler_descriptor)
        .build(device)?;


    let mut descriptor_sets = DescriptorSetAI::new(descriptor_pool)
        .add_set_layout(set_layout)
        .build(device)?;
    skybox.descriptor_set = descriptor_sets.remove(0);


    // Binding 0: Object matrices Uniform buffer.
    let ubo_write_info = DescriptorBufferSetWI::new(skybox.descriptor_set, 0, vk::DescriptorType::UNIFORM_BUFFER)
        .add_buffer(vk::DescriptorBufferInfo {
            buffer: skybox.ubo_buffer.handle,
            offset: 0,
            range : mem::size_of::<[UBOVS; 1]>() as vkbytes,
        });
    // Binding 1: Node hierarchy transform matrix in glTF.
    let node_write_info = DescriptorBufferSetWI::new(skybox.descriptor_set, 1, vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
        .add_buffer(skybox.model.nodes.node_descriptor());
    // Binding 2: Object texture.
    let sampler_write_info = DescriptorImageSetWI::new(skybox.descriptor_set, 2, vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .add_image(skybox.texture.descriptor());

    DescriptorSetsUpdateCI::new()
        .add_write(ubo_write_info.value())
        .add_write(node_write_info.value())
        .add_write(sampler_write_info.value())
        .update(device);

    let descriptors = DescriptorStaff {
        pool   : descriptor_pool,
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
        .add_color_attachment(0, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .set_depth_stencil_attachment(1, vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

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
    skybox : vk::Pipeline,
    _reflect: vk::Pipeline,
    layout: vk::PipelineLayout,
}

fn prepare_pipelines(device: &VkDevice, skybox: &Skybox, render_pass: vk::RenderPass, set_layout: vk::DescriptorSetLayout) -> VkResult<PipelineStaff> {

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

    // disable depth test for Skybox pipeline.
    let depth_stencil_state = DepthStencilSCI::new()
        .depth_test(false, false, vk::CompareOp::LESS_OR_EQUAL);

    let dynamic_state = DynamicSCI::new()
        .add_dynamic(vk::DynamicState::VIEWPORT)
        .add_dynamic(vk::DynamicState::SCISSOR);

    // Pipeline Layout.
    let pipeline_layout = PipelineLayoutCI::new()
        .add_set_layout(set_layout)
        .build(device)?;

    // shaders
    use vkbase::ci::shader::{ShaderModuleCI, ShaderStageCI};

    let mut shader_compiler = vkbase::utils::shaderc::VkShaderCompiler::new()?;
    let vert_codes = shader_compiler.compile_from_path(Path::new(SKY_BOX_VERTEX_SHADER_SOURCE_PATH), shaderc::ShaderKind::Vertex, "[Vertex Shader]", "main")?;
    let frag_codes = shader_compiler.compile_from_path(Path::new(SKY_BOX_FRAGMENT_SHADER_SOURCE_PATH), shaderc::ShaderKind::Fragment, "[Fragment Shader]", "main")?;

    let vert_module = ShaderModuleCI::from_glsl(vk::ShaderStageFlags::VERTEX, vert_codes)
        .build(device)?;
    let frag_module = ShaderModuleCI::from_glsl(vk::ShaderStageFlags::FRAGMENT, frag_codes)
        .build(device)?;

    // Pipeline.
    let mut pipeline_ci = GraphicsPipelineCI::new(render_pass, pipeline_layout);

    pipeline_ci.set_shaders(vec![
        ShaderStageCI::new(vk::ShaderStageFlags::VERTEX, vert_module),
        ShaderStageCI::new(vk::ShaderStageFlags::FRAGMENT, frag_module),
    ]);

    pipeline_ci.set_vertex_input(skybox.model.meshes.vertex_input.clone());
    pipeline_ci.set_viewport(viewport_state);
    pipeline_ci.set_depth_stencil(depth_stencil_state);
    pipeline_ci.set_rasterization(rasterization_state);
    pipeline_ci.set_color_blend(blend_state);
    pipeline_ci.set_dynamic(dynamic_state);

    // skybox pipeline (background cube).
    let skybox_pipeline = device.build(&pipeline_ci)?;

    // Destroy shader module.
    device.discard(vert_module);
    device.discard(frag_module);

    let result = PipelineStaff {
        skybox: skybox_pipeline,
        _reflect: vk::Pipeline::null(),
        layout: pipeline_layout,
    };
    Ok(result)
}

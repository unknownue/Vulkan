
use ash::vk;

use std::ptr;
use std::mem;
use std::path::Path;

use vkbase::context::{VkDevice, VkSwapchain};
use vkbase::ci::VkObjectBuildableCI;
use vkbase::ci::buffer::BufferCI;
use vkbase::ci::vma::{VmaBuffer, VmaAllocationCI};
use vkbase::ci::shader::{ShaderModuleCI, ShaderStageCI};
use vkbase::gltf::VkglTFModel;
use vkbase::ui::{TextInfo, TextType, TextHAlign};
use vkbase::context::VulkanContext;
use vkbase::utils::color::VkColor;
use vkbase::{FlightCamera, FrameAction};
use vkbase::{vkbytes, vkptr, Point3F, Matrix4F, Vector4F};
use vkbase::{VkResult, VkErrorKind};

use vkexamples::VkExampleBackend;

const PHONG_VERTEX_SHADER_SOURCE_PATH      : &'static str = "examples/src/pipelines/phong.vert.glsl";
const PHONG_FRAGMENT_SHADER_SOURCE_PATH    : &'static str = "examples/src/pipelines/phong.frag.glsl";
const TOON_VERTEX_SHADER_SOURCE_PATH       : &'static str = "examples/src/pipelines/toon.vert.glsl";
const TOON_FRAGMENT_SHADER_SOURCE_PATH     : &'static str = "examples/src/pipelines/toon.frag.glsl";
const WIREFRAME_VERTEX_SHADER_SOURCE_PATH  : &'static str = "examples/src/pipelines/wireframe.vert.glsl";
const WIREFRAME_FRAGMENT_SHADER_SOURCE_PATH: &'static str = "examples/src/pipelines/wireframe.frag.glsl";
const MODEL_PATH: &'static str = "assets/models/treasure_smooth.gltf";


pub struct VulkanExample {

    backend: VkExampleBackend,

    model: VkglTFModel,
    uniform_buffer: VmaBuffer,

    pipelines: PipelineStaff,
    descriptors: DescriptorStaff,

    ubo_data: UboVS,
    camera: FlightCamera,

    is_toggle_event: bool,
}

struct PipelineStaff {
    phong     : vk::Pipeline,
    wireframe : vk::Pipeline,
    toon      : vk::Pipeline,
    layout: vk::PipelineLayout,
}

impl VulkanExample {

    pub fn new(context: &mut VulkanContext) -> VkResult<VulkanExample> {

        let device = &mut context.device;
        let swapchain = &context.swapchain;
        let dimension = swapchain.dimension;

        let mut camera = FlightCamera::new()
            .place_at(Point3F::new(0.25, 6.25, 8.75))
            .screen_aspect_ratio((dimension.width as f32 / 3.0) / dimension.height as f32)
            .pitch(-45.0)
            .build();
        camera.set_move_speed(50.0);

        let ubo_data = UboVS {
            projection : camera.proj_matrix(),
            view       : camera.view_matrix(),
            model      : Matrix4F::identity(),
            light_pos  : Vector4F::new(0.0, 2.0, 1.0, 0.0),
        };

        let render_pass = setup_renderpass(device, &context.swapchain)?;
        let backend = VkExampleBackend::new(device, swapchain, render_pass)?;

        let model = prepare_model(device)?;
        let uniform_buffer = prepare_uniform(device, &ubo_data)?;
        let descriptors = setup_descriptor(device, &uniform_buffer, &model)?;

        let pipelines = prepare_pipelines(device, &model, backend.render_pass, descriptors.layout)?;

        let target = VulkanExample {
            backend, model, uniform_buffer, descriptors, pipelines, camera, ubo_data,
            is_toggle_event: false,
        };
        Ok(target)
    }
}

impl vkbase::RenderWorkflow for VulkanExample {

    fn init(&mut self, device: &VkDevice) -> VkResult<()> {

        self.backend.set_basic_ui(device, super::WINDOW_TITLE)?;

        let screen_width  = self.backend.dimension.width  as i32;
        let screen_height = self.backend.dimension.height as i32;

        let phong_text = TextInfo {
            content: String::from("Phong Shading Pipeline"),
            scale: 16.0,
            align: TextHAlign::Center,
            color: VkColor::WHITE,
            location: vk::Offset2D { x: screen_width / 6, y: screen_height / 8 * 7 },
            r#type: TextType::Static,
        };
        self.backend.ui_renderer.add_text(phong_text)?;

        let toon_text = TextInfo {
            content: String::from("Toon Shading Pipeline"),
            scale: 16.0,
            align: TextHAlign::Center,
            color: VkColor::WHITE,
            location: vk::Offset2D { x: screen_width / 6 * 3, y: screen_height / 8 * 7 },
            r#type: TextType::Static,
        };
        self.backend.ui_renderer.add_text(toon_text)?;

        let wireframe_text = TextInfo {
            content: String::from("Wireframe Pipeline"),
            scale: 16.0,
            align: TextHAlign::Center,
            color: VkColor::WHITE,
            location: vk::Offset2D { x: screen_width / 6 * 5 , y: screen_height / 8 * 7 },
            r#type: TextType::Static,
        };
        self.backend.ui_renderer.add_text(wireframe_text)?;

        self.record_commands(device, self.backend.dimension)?;

        Ok(())
    }

    fn render_frame(&mut self, device: &mut VkDevice, device_available: vk::Fence, await_present: vk::Semaphore, image_index: usize, _delta_time: f32) -> VkResult<vk::Semaphore> {

        if self.is_toggle_event {
            self.update_uniforms()?;
        }

        let submit_ci = vkbase::ci::device::SubmitCI::new()
            .add_wait(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, await_present)
            .add_command(self.backend.commands[image_index])
            .add_signal(self.backend.await_rendering);

        // Submit to the graphics queue passing a wait fence.
        device.submit(submit_ci, device.logic.queues.graphics.handle, Some(device_available))?;

        Ok(self.backend.await_rendering)
    }

    fn swapchain_reload(&mut self, device: &mut VkDevice, new_chain: &VkSwapchain) -> VkResult<()> {

        // recreate the resources.
        device.discard(self.pipelines.phong);
        device.discard(self.pipelines.toon);
        device.discard(self.pipelines.wireframe);

        let render_pass = setup_renderpass(device, new_chain)?;
        self.backend.swapchain_reload(device, new_chain, render_pass)?;
        self.pipelines = prepare_pipelines(device, &self.model, self.backend.render_pass, self.descriptors.layout)?;

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

        device.discard(self.pipelines.phong);
        device.discard(self.pipelines.toon);
        device.discard(self.pipelines.wireframe);
        device.discard(self.pipelines.layout);

        device.vma_discard(self.uniform_buffer)?;
        device.vma_discard(self.model)?;
        self.backend.discard_by(device)
    }
}

impl VulkanExample {

    fn record_commands(&self, device: &VkDevice, dimension: vk::Extent2D) -> VkResult<()> {

        let scissor = vk::Rect2D {
            extent: dimension.clone(),
            offset: vk::Offset2D { x: 0, y: 0 },
        };

        for (i, &command) in self.backend.commands.iter().enumerate() {

            use vkbase::command::{VkCmdRecorder, CmdGraphicsApi, IGraphics};
            use vkbase::ci::pipeline::RenderPassBI;

            let render_params = vkbase::gltf::ModelRenderParams {
                descriptor_set : self.descriptors.set,
                pipeline_layout: self.pipelines.layout,
                material_stage : Some(vk::ShaderStageFlags::VERTEX),
            };

            let mut viewport = vk::Viewport {
                x: 0.0, y: 0.0,
                width: dimension.width as f32, height: dimension.height as f32,
                min_depth: 0.0, max_depth: 1.0,
            };

            let recorder: VkCmdRecorder<IGraphics> = VkCmdRecorder::new(&device.logic, command);

            let render_pass_bi = RenderPassBI::new(self.backend.render_pass, self.backend.framebuffers[i])
                .render_extent(dimension)
                .set_clear_values(vkexamples::DEFAULT_CLEAR_VALUES.clone());

            recorder.begin_record()?
                .begin_render_pass(render_pass_bi)
                .set_scissor(0, &[scissor]);

            { // Left: Solid colored
                viewport.width = dimension.width as f32 / 3.0;
                recorder
                    .set_viewport(0, &[viewport])
                    .bind_pipeline(self.pipelines.phong);
                self.model.record_command(&recorder, &render_params);
            }

            { // Center: Toon
                viewport.x = dimension.width as f32 / 3.0;
                recorder
                    .set_viewport(0, &[viewport])
                    .bind_pipeline(self.pipelines.toon);

                // Line width > 1.0f only if wide lines feature is supported.
                if device.phy.features_enabled().wide_lines == vk::TRUE {
                    recorder.set_line_width(2.0);
                }
                self.model.record_command(&recorder, &render_params);
            }

            { // Right: Wireframe
                if device.phy.features_enabled().fill_mode_non_solid == vk::TRUE {
                    viewport.x = dimension.width as f32 / 3.0 * 2.0;
                    recorder
                        .set_viewport(0, &[viewport])
                        .bind_pipeline(self.pipelines.wireframe);
                    self.model.record_command(&recorder, &render_params);
                }
            }

            self.backend.ui_renderer.record_command(&recorder);

            recorder
                .end_render_pass()
                .end_record()?;
        }

        Ok(())
    }

    fn update_uniforms(&mut self) -> VkResult<()> {

        self.ubo_data.view = self.camera.view_matrix();

        unsafe {
            let data_ptr = self.uniform_buffer.info.get_mapped_data() as vkptr<UboVS>;
            data_ptr.copy_from_nonoverlapping(&self.ubo_data, 1);
        }

        Ok(())
    }
}

// Prepare model from glTF file.
pub fn prepare_model(device: &mut VkDevice) -> VkResult<VkglTFModel> {

    use vkbase::gltf::{GltfModelInfo, load_gltf};
    use vkbase::gltf::{AttributeFlags, NodeAttachmentFlags};

    let model_info = GltfModelInfo {
        path: Path::new(MODEL_PATH),
        attribute: AttributeFlags::POSITION | AttributeFlags::NORMAL, // specify model's vertices layout.
        node: NodeAttachmentFlags::TRANSFORM_MATRIX, // specify model's node attachment layout.
        transform: None,
    };

    let model = load_gltf(device, model_info)?;
    Ok(model)
}


// The uniform data that will be transferred to shader.
//
// layout (set = 0, binding = 0) uniform UBO {
//     mat4 projection;
//     mat4 view;
//     mat4 model;
//     vec4 lightPos;
// } ubo;
#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct UboVS {
    projection   : Matrix4F,
    view         : Matrix4F,
    model        : Matrix4F,
    light_pos    : Vector4F,
}

fn prepare_uniform(device: &mut VkDevice, ubo_data: &UboVS) -> VkResult<VmaBuffer> {

    let uniform_buffer = {
        let uniform_ci = BufferCI::new(mem::size_of::<UboVS>() as vkbytes)
            .usage(vk::BufferUsageFlags::UNIFORM_BUFFER);
        let allocation_ci = VmaAllocationCI::new(vma::MemoryUsage::CpuOnly, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)
            .flags(vma::AllocationCreateFlags::MAPPED);
        let uniform_allocation = device.vma.create_buffer(uniform_ci.as_ref(), allocation_ci.as_ref())
            .map_err(VkErrorKind::Vma)?;
        VmaBuffer::from(uniform_allocation)
    };

    // keep the uniform memory map during the program running.
    unsafe {
        let data_ptr = uniform_buffer.info.get_mapped_data() as vkptr<UboVS>;
        debug_assert_ne!(data_ptr, ptr::null_mut());
        data_ptr.copy_from_nonoverlapping(ubo_data, 1);
    }

    Ok(uniform_buffer)
}

struct DescriptorStaff {
    pool   : vk::DescriptorPool,
    set    : vk::DescriptorSet,
    layout : vk::DescriptorSetLayout,
}

fn setup_descriptor(device: &VkDevice, uniform_buffer: &VmaBuffer, model: &VkglTFModel) -> VkResult<DescriptorStaff> {

    use vkbase::ci::descriptor::{DescriptorPoolCI, DescriptorSetLayoutCI};
    use vkbase::ci::descriptor::{DescriptorSetAI, DescriptorBufferSetWI, DescriptorSetsUpdateCI};

    // Descriptor Pool.
    let descriptor_pool = DescriptorPoolCI::new(1)
        .add_descriptor(vk::DescriptorType::UNIFORM_BUFFER, 1)
        .add_descriptor(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC, 1)
        .build(device)?;

    // in phone.vert.glsl:
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

    // in phone.vert.glsl:
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
        .add_buffer(vk::DescriptorBufferInfo {
            buffer: uniform_buffer.handle,
            offset: 0,
            range : mem::size_of::<UboVS>() as vkbytes,
        });
    let node_write_info = DescriptorBufferSetWI::new(descriptor_set, 1, vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
        .add_buffer(model.nodes.node_descriptor());

    DescriptorSetsUpdateCI::new()
        .add_write(&ubo_write_info)
        .add_write(&node_write_info)
        .update(device);

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
        .add_attachment(color_attachment)
        .add_attachment(depth_attachment)
        .add_subpass(subpass_description)
        .add_dependency(dependency0)
        .add_dependency(dependency1)
        .build(device)?;

    Ok(render_pass)
}

fn prepare_pipelines(device: &VkDevice, model: &VkglTFModel, render_pass: vk::RenderPass, set_layout: vk::DescriptorSetLayout) -> VkResult<PipelineStaff> {

    use vkbase::ci::pipeline::*;

    let viewport_state = ViewportSCI::new()
        .add_viewport(vk::Viewport::default())
        .add_scissor(vk::Rect2D::default());

    let mut rasterization_state = RasterizationSCI::new()
        .polygon(vk::PolygonMode::FILL)
        .cull_face(vk::CullModeFlags::BACK, vk::FrontFace::CLOCKWISE);

    let blend_attachment = BlendAttachmentSCI::new();
    let blend_state = ColorBlendSCI::new()
        .add_attachment(blend_attachment);

    let depth_stencil_state = DepthStencilSCI::new()
        .depth_test(true, true, vk::CompareOp::LESS_OR_EQUAL);

    let mut dynamic_state = DynamicSCI::new()
        .add_dynamic(vk::DynamicState::VIEWPORT)
        .add_dynamic(vk::DynamicState::SCISSOR);

    if device.phy.features_enabled().wide_lines == vk::TRUE {
        dynamic_state = dynamic_state.add_dynamic(vk::DynamicState::LINE_WIDTH)
    };

    let material_range = vk::PushConstantRange {
        stage_flags: vk::ShaderStageFlags::VERTEX,
        offset: 0,
        size: model.materials.material_size(),
    };

    // Pipeline Layout.
    let pipeline_layout = PipelineLayoutCI::new()
        .add_set_layout(set_layout)
        .add_push_constants(material_range)
        .build(device)?;

    // base pipeline.
    let mut pipeline_ci = GraphicsPipelineCI::new(render_pass, pipeline_layout);

    pipeline_ci.set_vertex_input(model.meshes.vertex_input.clone());
    pipeline_ci.set_viewport(viewport_state);
    pipeline_ci.set_rasterization(rasterization_state.clone());
    pipeline_ci.set_depth_stencil(depth_stencil_state);
    pipeline_ci.set_color_blend(blend_state);
    pipeline_ci.set_dynamic(dynamic_state);


    let mut shader_compiler = vkbase::utils::shaderc::VkShaderCompiler::new()?;

    let phong_pipeline = {

        let vert_codes = shader_compiler.compile_from_path(Path::new(PHONG_VERTEX_SHADER_SOURCE_PATH), shaderc::ShaderKind::Vertex, "[Vertex Shader]", "main")?;
        let frag_codes = shader_compiler.compile_from_path(Path::new(PHONG_FRAGMENT_SHADER_SOURCE_PATH), shaderc::ShaderKind::Fragment, "[Fragment Shader]", "main")?;

        let vert_module = ShaderModuleCI::new(vert_codes)
            .build(device)?;
        let frag_module = ShaderModuleCI::new(frag_codes).build(device)?;

        let shaders = [
            ShaderStageCI::new(vk::ShaderStageFlags::VERTEX, vert_module),
            ShaderStageCI::new(vk::ShaderStageFlags::FRAGMENT, frag_module),
        ];
        pipeline_ci.set_shaders(&shaders);

        // Using this pipeline as the base for the other pipelines (derivatives).
        // Pipeline derivatives can be used for pipelines that share most of their state
        // depending on the implementation this may result in better performance for pipeline switching and faster creation time.
        pipeline_ci.set_flags(vk::PipelineCreateFlags::ALLOW_DERIVATIVES);

        let pipeline = device.build(&pipeline_ci)?;

        device.discard(vert_module);
        device.discard(frag_module);

        pipeline
    };

    let toon_pipeline = {

        let vert_codes = shader_compiler.compile_from_path(Path::new(TOON_VERTEX_SHADER_SOURCE_PATH), shaderc::ShaderKind::Vertex, "[Vertex Shader]", "main")?;
        let frag_codes = shader_compiler.compile_from_path(Path::new(TOON_FRAGMENT_SHADER_SOURCE_PATH), shaderc::ShaderKind::Fragment, "[Fragment Shader]", "main")?;

        let vert_module = ShaderModuleCI::new(vert_codes)
            .build(device)?;
        let frag_module = ShaderModuleCI::new(frag_codes).build(device)?;

        let shaders = [
            ShaderStageCI::new(vk::ShaderStageFlags::VERTEX, vert_module),
            ShaderStageCI::new(vk::ShaderStageFlags::FRAGMENT, frag_module),
        ];
        pipeline_ci.set_shaders(&shaders);
        // Base pipeline will be our first created pipeline.
        pipeline_ci.set_base_pipeline(phong_pipeline);
        // All pipelines created after the base pipeline will be derivatives.
        pipeline_ci.set_flags(vk::PipelineCreateFlags::DERIVATIVE);

        let pipeline = device.build(&pipeline_ci)?;

        device.discard(vert_module);
        device.discard(frag_module);

        pipeline
    };

    let wireframe_pipeline = {

        let vert_codes = shader_compiler.compile_from_path(Path::new(WIREFRAME_VERTEX_SHADER_SOURCE_PATH), shaderc::ShaderKind::Vertex, "[Vertex Shader]", "main")?;
        let frag_codes = shader_compiler.compile_from_path(Path::new(WIREFRAME_FRAGMENT_SHADER_SOURCE_PATH), shaderc::ShaderKind::Fragment, "[Fragment Shader]", "main")?;

        let vert_module = ShaderModuleCI::new(vert_codes)
            .build(device)?;
        let frag_module = ShaderModuleCI::new(frag_codes).build(device)?;

        let shaders = [
            ShaderStageCI::new(vk::ShaderStageFlags::VERTEX, vert_module),
            ShaderStageCI::new(vk::ShaderStageFlags::FRAGMENT, frag_module),
        ];
        pipeline_ci.set_shaders(&shaders);

        // Non solid rendering is not a mandatory Vulkan feature.
        if device.phy.features_enabled().fill_mode_non_solid == vk::TRUE {
            rasterization_state = rasterization_state.polygon(vk::PolygonMode::LINE);
            pipeline_ci.set_rasterization(rasterization_state);
        }

        let pipeline = device.build(&pipeline_ci)?;

        device.discard(vert_module);
        device.discard(frag_module);

        pipeline
    };


    let result = PipelineStaff {
        phong: phong_pipeline,
        toon : toon_pipeline,
        wireframe: wireframe_pipeline,

        layout: pipeline_layout,
    };
    Ok(result)
}

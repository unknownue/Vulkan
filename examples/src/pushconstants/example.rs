
use ash::vk;

use std::ptr;
use std::mem;
use std::path::Path;

use vkbase::context::{VkDevice, VkSwapchain};
use vkbase::ci::VkObjectBuildableCI;
use vkbase::ci::buffer::BufferCI;
use vkbase::ci::vma::{VmaBuffer, VmaAllocationCI};
use vkbase::gltf::VkglTFModel;
use vkbase::context::VulkanContext;
use vkbase::{FlightCamera, FrameAction};
use vkbase::{vkbytes, vkuint, vkptr, Point3F, Matrix4F};
use vkbase::{VkResult, VkErrorKind};

use vkexamples::VkExampleBackendRes;

const VERTEX_SHADER_SOURCE_PATH  : &'static str = "examples/src/pushconstants/lights.vert.glsl";
const FRAGMENT_SHADER_SOURCE_PATH: &'static str = "examples/src/pushconstants/lights.frag.glsl";
const MODEL_PATH: &'static str = "assets/models/samplescene.gltf";
const TIMER: f32 = 0.10;

pub struct VulkanExample {

    backend: VkExampleBackendRes,

    model: VkglTFModel,

    ubo_buffer: VmaBuffer,
    ubo_data: [UBOVS; 1],

    pipelines: PipelineStaff,
    descriptors: DescriptorStaff,

    camera: FlightCamera,

    is_toggle_event: bool,
}

/// The data structure of push constant block.
/// in lights.vert.glsl:
///
/// layout(push_constant) uniform PushConsts {
///	    vec4 lightPos[lightCount];
/// } pushConsts;
#[derive(Debug, Clone)]
#[repr(C)]
struct PushConstants {
    lights: [[f32; 4]; 6],
}

impl VulkanExample {

    pub fn new(context: &mut VulkanContext) -> VkResult<VulkanExample> {

        let device = &mut context.device;
        let swapchain = &context.swapchain;
        let dimension = swapchain.dimension;

        let mut camera = FlightCamera::new()
            .place_at(Point3F::new(-11.0, 45.0, 26.0))
            .screen_aspect_ratio(dimension.width as f32 / dimension.height as f32)
            .pitch(-45.0)
            .yaw(-45.0)
            .build();
        camera.set_move_speed(50.0);

        let ubo_data = [
            UBOVS {
                projection: camera.proj_matrix(),
                view      : camera.view_matrix(),
                model     : Matrix4F::identity(),
            },
        ];


        let render_pass = setup_renderpass(device, &context.swapchain)?;
        let backend = VkExampleBackendRes::new(device, swapchain, render_pass)?;

        let model = prepare_model(device)?;

        let ubo_buffer = prepare_uniform(device)?;
        let descriptors = setup_descriptor(device, &ubo_buffer, &model)?;

        let pipelines = prepare_pipelines(device, &model, backend.render_pass, descriptors.layout)?;

        let target = VulkanExample {
            backend, model, ubo_buffer, ubo_data, descriptors, pipelines, camera,
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

        self.update_uniforms(delta_time)?;

        let submit_ci = vkbase::ci::device::SubmitCI::new()
            .add_wait(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, await_present)
            .add_command(self.backend.commands[image_index])
            .add_signal(self.backend.await_rendering);

        device.submit(submit_ci, device.logic.queues.graphics.handle, device_available)?;

        Ok(self.backend.await_rendering)
    }

    fn swapchain_reload(&mut self, device: &mut VkDevice, new_chain: &VkSwapchain) -> VkResult<()> {

        device.discard(self.pipelines.pipeline);

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

        device.discard(self.pipelines.pipeline);
        device.discard(self.pipelines.layout);

        device.vma_discard(self.ubo_buffer)?;
        device.vma_discard(self.model)?;
        self.backend.discard_by(device)
    }
}

impl VulkanExample {

    fn pubconstant_data() -> PushConstants {

        const R : f32 = 10.5;
        const Y1: f32 = -2.0;
        const Y2: f32 = 15.0;

        let sin_t = (TIMER * 360.0).to_radians().sin();
        let cos_t = (TIMER * 360.0).to_radians().cos();

        PushConstants {
            // w component = light radius scale.
            lights: [
                [R * 1.1 * sin_t, Y1, R * 1.1 * cos_t, 2.0],
                [-R * sin_t, Y1, -R * cos_t, 2.0],
                [R * 0.85 * sin_t, Y1, -sin_t * 2.5, 3.0],
                [0.0, Y2, R * 1.25 * cos_t, 3.0],
                [R * 2.25 * cos_t, Y2, 0.0, 2.5],
                [R * 2.5 * cos_t, Y2, R * 2.5 * sin_t, 2.5],
            ],
        }
    }

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

            let push_data = VulkanExample::pubconstant_data();
            let push_data_ptr = unsafe {
                vkbase::utils::memory::any_as_u8_slice(&push_data)
            };

            let recorder: VkCmdRecorder<IGraphics> = VkCmdRecorder::new(&device.logic, command);

            let render_pass_bi = RenderPassBI::new(self.backend.render_pass, self.backend.framebuffers[i])
                .render_extent(dimension)
                .clear_values(&clear_values);

            recorder.begin_record()?
                .begin_render_pass(render_pass_bi)
                .set_viewport(0, &[viewport])
                .set_scissor(0, &[scissor])
                .bind_pipeline(self.pipelines.pipeline)
                .push_constants(self.pipelines.layout, vk::ShaderStageFlags::VERTEX, 0, push_data_ptr);

            let render_params = vkbase::gltf::ModelRenderParams {
                descriptor_set : self.descriptors.set,
                pipeline_layout: self.pipelines.layout,
                material_stage : None,
            };

            self.model.record_command(&recorder, &render_params);

            self.backend.ui_renderer.record_command(&recorder);

            recorder
                .end_render_pass()
                .end_record()?;
        }

        Ok(())
    }

    fn update_uniforms(&mut self, _delta_time: f32) -> VkResult<()> {
        
        if self.is_toggle_event {
            
            self.ubo_data[0].view = self.camera.view_matrix();

            use vkbase::utils::memory::copy_to_ptr;
            copy_to_ptr(self.ubo_buffer.info.get_mapped_data() as vkptr, &self.ubo_data);
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
        // specify model's vertices layout.
        // in light.vert.glsl:
        // layout (location = 0) in vec3 inPos;
        // layout (location = 1) in vec3 inNormal;
        attribute: AttributeFlags::POSITION | AttributeFlags::NORMAL,
        // specify model's node attachment layout.
        // in light.vert.glsl
        // layout (set = 0, binding = 1) uniform DynNode {
        //     mat4 transform;
        // } dyn_node;
        node: NodeAttachmentFlags::TRANSFORM_MATRIX,
    };

    let model = load_gltf(device, model_info)?;
    Ok(model)
}


/// The uniform structure used in shader.
///
/// layout (set = 0, binding = 0) uniform UBO {
///     mat4 projection;
///     mat4 view;
///     mat4 model;
/// } ubo;
#[derive(Debug, Clone)]
#[repr(C)]
struct UBOVS {
    projection: Matrix4F,
    view      : Matrix4F,
    model     : Matrix4F,
}

fn prepare_uniform(device: &mut VkDevice) -> VkResult<VmaBuffer> {

    let uniform_buffer = {

        let uniform_ci = BufferCI::new(mem::size_of::<UBOVS>() as vkbytes)
            .usage(vk::BufferUsageFlags::UNIFORM_BUFFER);
        let allocation_ci = VmaAllocationCI::new(vma::MemoryUsage::CpuOnly, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)
            .flags(vma::AllocationCreateFlags::MAPPED);
        let uniform_allocation = device.vma.create_buffer(&uniform_ci.value(), allocation_ci.as_ref())
            .map_err(VkErrorKind::Vma)?;

        VmaBuffer::from(uniform_allocation)
    };

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

    // in light.vert.glsl:
    //
    // layout (set = 0, binding = 0) uniform UBO {
    //     mat4 projection;
    //     mat4 view;
    //     mat4 model;
    // } ubo;
    let ubo_descriptor = vk::DescriptorSetLayoutBinding {
        binding: 0,
        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
        descriptor_count: 1,
        stage_flags: vk::ShaderStageFlags::VERTEX,
        p_immutable_samplers: ptr::null(),
    };

    // in light.vert.glsl:
    //
    // layout (set = 0, binding = 1) uniform DynNode {
    //     mat4 transform;
    // } dyn_node;
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
            range : mem::size_of::<UBOVS>() as vkbytes,
        });
    let node_write_info = DescriptorBufferSetWI::new(descriptor_set, 1, vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
        .add_buffer(model.nodes.node_descriptor());

    DescriptorSetsUpdateCI::new()
        .add_write(ubo_write_info.value())
        .add_write(node_write_info.value())
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
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
}

fn prepare_pipelines(device: &VkDevice, model: &VkglTFModel, render_pass: vk::RenderPass, set_layout: vk::DescriptorSetLayout) -> VkResult<PipelineStaff> {

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

    // --------------------------------------------------------------------------------------
    // Sascha Willems's comment:
    //
    // Define push constant
    //
    // Example uses six light positions as push constants
    // 6 * 4 * 4 = 96 bytes
    // Spec requires a minimum of 128 bytes, bigger values need to be checked against maxPushConstantsSize.
    // But even at only 128 bytes, lots of stuff can fit inside push constants.
    //

    let push_constant_range = vk::PushConstantRange {
        stage_flags: vk::ShaderStageFlags::VERTEX,
        offset: 0,
        size: ::std::mem::size_of::<PushConstants>() as vkuint,
    };

    // Pipeline Layout.
    let layout = PipelineLayoutCI::new()
        .add_set_layout(set_layout)
        // Push constant ranges are part of the pipeline layout.
        .add_push_constants(push_constant_range)
        .build(device)?;
    // ---------------------------------------------------------------------------------------

    // shaders
    use vkbase::ci::shader::{ShaderModuleCI, ShaderStageCI};

    let mut shader_compiler = vkbase::utils::shaderc::VkShaderCompiler::new()?;
    let vert_codes = shader_compiler.compile_from_path(Path::new(VERTEX_SHADER_SOURCE_PATH), shaderc::ShaderKind::Vertex, "[Vertex Shader]", "main")?;
    let frag_codes = shader_compiler.compile_from_path(Path::new(FRAGMENT_SHADER_SOURCE_PATH), shaderc::ShaderKind::Fragment, "[Fragment Shader]", "main")?;

    let vert_module = ShaderModuleCI::from_glsl(vk::ShaderStageFlags::VERTEX, vert_codes)
        .build(device)?;
    let frag_module = ShaderModuleCI::from_glsl(vk::ShaderStageFlags::FRAGMENT, frag_codes)
        .build(device)?;

    // Pipeline.
    let mut pipeline_ci = GraphicsPipelineCI::new(render_pass, layout);

    pipeline_ci.set_shaders(vec![
        ShaderStageCI::new(vk::ShaderStageFlags::VERTEX, vert_module),
        ShaderStageCI::new(vk::ShaderStageFlags::FRAGMENT, frag_module),
    ]);

    pipeline_ci.set_vertex_input(model.meshes.vertex_input.clone());
    pipeline_ci.set_viewport(viewport_state);
    pipeline_ci.set_depth_stencil(depth_stencil_state);
    pipeline_ci.set_rasterization(rasterization_state);
    pipeline_ci.set_color_blend(blend_state);
    pipeline_ci.set_dynamic(dynamic_state);

    let pipeline = device.build(&pipeline_ci)?;

    // Destroy shader module.
    device.discard(vert_module);
    device.discard(frag_module);

    let result = PipelineStaff { pipeline, layout };
    Ok(result)
}

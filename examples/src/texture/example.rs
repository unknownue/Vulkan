
use ash::vk;

use std::ptr;
use std::path::Path;

use vkbase::context::{VulkanContext, VkDevice, VkSwapchain};
use vkbase::ci::VkObjectBuildableCI;
use vkbase::ci::vma::VmaBuffer;
use vkbase::utils::color::VkColor;
use vkbase::ui::{TextInfo, TextType, TextHAlign};
use vkbase::{FlightCamera, FrameAction};
use vkbase::{vkuint, vkptr, Point3F, Point4F};
use vkbase::VkResult;

use vkexamples::VkExampleBackend;
use crate::data::{INDEX_DATA, Vertex, UboVS, Texture};

const SHADER_VERTEX_PATH  : &'static str = "examples/src/texture/texture.vert.glsl";
const SHADER_FRAGMENT_PATH: &'static str = "examples/src/texture/texture.frag.glsl";
const TEXTURE_PATH: &'static str = "assets/textures/metalplate01_rgba.ktx";

pub struct VulkanExample {

    backend: VkExampleBackend,
    camera: FlightCamera,

    vertices: VmaBuffer,
    indices : VmaBuffer,

    ubo_buffer: VmaBuffer,
    ubo_data: UboVS,

    texture: Texture,

    pipelines: PipelineStaff,
    descriptors: DescriptorStaff,

    lod_text_id: usize,
    is_toggle_event: bool,
}

impl VulkanExample {

    pub fn new(context: &mut VulkanContext) -> VkResult<VulkanExample> {

        let device = &mut context.device;
        let swapchain = &context.swapchain;
        let dimension = swapchain.dimension;

        let mut camera = FlightCamera::new()
            .place_at(Point3F::new(0.0, 0.0, 2.5))
            .screen_aspect_ratio(dimension.width as f32 / dimension.height as f32)
            .build();
        camera.set_move_speed(5.0);

        let render_pass = setup_renderpass(device, &context.swapchain)?;
        let backend = VkExampleBackend::new(device, swapchain, render_pass)?;

        let (vertices, indices) = super::data::generate_quad(device)?;
        let (ubo_buffer, ubo_data) = UboVS::prepare_buffer(device, &camera)?;
        let texture = Texture::load_ktx(device, Path::new(TEXTURE_PATH))?;

        let descriptors = setup_descriptor(device, &ubo_buffer, &texture)?;

        let pipelines = prepare_pipelines(device, backend.render_pass, descriptors.layout)?;

        let target = VulkanExample {
            backend, descriptors, pipelines, camera,
            vertices, indices, texture,
            ubo_buffer, ubo_data,
            lod_text_id: 0,
            is_toggle_event: true,
        };
        Ok(target)
    }
}

impl vkbase::RenderWorkflow for VulkanExample {

    fn init(&mut self, device: &VkDevice) -> VkResult<()> {

        self.backend.set_basic_ui(device, super::WINDOW_TITLE)?;

        let lod_text = TextInfo {
            content: format!("Lod bias: {:1.2} (numpad +/- to change)", self.ubo_data.lod_bias),
            scale: 14.0,
            align: TextHAlign::Left,
            color: VkColor::WHITE,
            location: vk::Offset2D { x: 5, y: 140 },
            r#type: TextType::Dynamic { capacity: 40 },
        };
        self.lod_text_id = self.backend.ui_renderer.add_text(lod_text)?;

        self.update_uniforms()?;
        self.record_commands(device, self.backend.dimension)?;
        Ok(())
    }

    fn render_frame(&mut self, device: &mut VkDevice, device_available: vk::Fence, await_present: vk::Semaphore, image_index: usize, _delta_time: f32) -> VkResult<vk::Semaphore> {

        self.update_uniforms()?;

        let submit_ci = vkbase::ci::device::SubmitCI::new()
            .add_wait(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, await_present)
            .add_command(self.backend.commands[image_index])
            .add_signal(self.backend.await_rendering);

        device.submit(submit_ci, device.logic.queues.graphics.handle, Some(device_available))?;

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

            if inputer.key.is_key_pressed(winit::VirtualKeyCode::Equals) && self.ubo_data.lod_bias < self.texture.mip_levels as f32 {

                self.ubo_data.lod_bias += 0.05;
                self.backend.ui_renderer.change_text(
                    format!("Lod bias: {:1.2} (numpad +/- to change)", self.ubo_data.lod_bias),
                    self.lod_text_id);
            } else if inputer.key.is_key_pressed(winit::VirtualKeyCode::Minus) && self.ubo_data.lod_bias > 0.0 {

                self.ubo_data.lod_bias -= 0.05;
                self.backend.ui_renderer.change_text(
                    format!("Lod bias: {:1.2} (numpad +/- to change)", self.ubo_data.lod_bias),
                    self.lod_text_id);
            }
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
        device.vma_discard(self.vertices)?;
        device.vma_discard(self.indices)?;

        self.texture.discard_by(device)?;
        self.backend.discard_by(device)
    }
}

impl VulkanExample {

    fn record_commands(&self, device: &VkDevice, dimension: vk::Extent2D) -> VkResult<()> {

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
                .set_clear_values(vkexamples::DEFAULT_CLEAR_VALUES.clone());

            recorder.begin_record()?
                .begin_render_pass(render_pass_bi)
                .set_viewport(0, &[viewport])
                .set_scissor(0, &[scissor])
                .bind_pipeline(self.pipelines.pipeline)
                .bind_descriptor_sets(self.pipelines.layout, 0, &[self.descriptors.set], &[])
                .bind_vertex_buffers(0, &[self.vertices.handle], &[0])
                .bind_index_buffer(self.indices.handle, vk::IndexType::UINT32, 0)
                .draw_indexed(INDEX_DATA.len() as vkuint, 1, 0, 0, 0);

            self.backend.ui_renderer.record_command(&recorder);

            recorder.end_render_pass()
                .end_record()?;
        }

        Ok(())
    }

    fn update_uniforms(&mut self) -> VkResult<()> {

        if self.is_toggle_event {

            let camera_pos = self.camera.current_position();
            self.ubo_data.view_pos = Point4F::new(camera_pos.x, camera_pos.y, camera_pos.z, 0.0);
            self.ubo_data.model = self.camera.view_matrix();

            unsafe {
                let data_ptr = self.ubo_buffer.info.get_mapped_data() as vkptr<UboVS>;
                data_ptr.copy_from_nonoverlapping(&self.ubo_data, 1);
            }
        }

        Ok(())
    }
}



struct DescriptorStaff {
    pool   : vk::DescriptorPool,
    set    : vk::DescriptorSet,
    layout : vk::DescriptorSetLayout,
}

fn setup_descriptor(device: &VkDevice, ubo_buffer: &VmaBuffer, texture: &Texture) -> VkResult<DescriptorStaff> {

    use vkbase::ci::descriptor::{DescriptorPoolCI, DescriptorSetLayoutCI};
    use vkbase::ci::descriptor::{DescriptorSetAI, DescriptorBufferSetWI, DescriptorImageSetWI, DescriptorSetsUpdateCI};

    // Descriptor Pool.
    let descriptor_pool = DescriptorPoolCI::new(1)
        .add_descriptor(vk::DescriptorType::UNIFORM_BUFFER, 1)
        .add_descriptor(vk::DescriptorType::COMBINED_IMAGE_SAMPLER, 1)
        .build(device)?;

    // in texture.vert.glsl:
    // layout (set = 0, binding = 0) uniform UBO {
    //     mat4 projection;
    //     mat4 model;
    //     vec4 viewPos;
    //     float lodBias;
    // } ubo;
    let ubo_descriptor = vk::DescriptorSetLayoutBinding {
        binding: 0,
        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
        descriptor_count: 1,
        stage_flags: vk::ShaderStageFlags::VERTEX,
        p_immutable_samplers: ptr::null(),
    };

    // in texture.frag.glsl:
    // layout (binding = 1) uniform sampler2D samplerColor;
    let sampler_descriptor = vk::DescriptorSetLayoutBinding {
        binding: 1,
        descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        descriptor_count: 1,
        stage_flags: vk::ShaderStageFlags::FRAGMENT,
        p_immutable_samplers: ptr::null(),
    };

    let set_layout = DescriptorSetLayoutCI::new()
        .add_binding(ubo_descriptor)
        .add_binding(sampler_descriptor)
        .build(device)?;

    // Descriptor set.
    let mut descriptor_sets = DescriptorSetAI::new(descriptor_pool)
        .add_set_layout(set_layout)
        .build(device)?;
    let descriptor_set = descriptor_sets.remove(0);

    let ubo_write = DescriptorBufferSetWI::new(descriptor_set, 0, vk::DescriptorType::UNIFORM_BUFFER)
        .add_buffer(vk::DescriptorBufferInfo {
            buffer: ubo_buffer.handle,
            offset: 0,
            range : vk::WHOLE_SIZE,
        });

    // Setup a descriptor image info for the current texture to be used as a combined image sampler.
    let sampler_write = DescriptorImageSetWI::new(descriptor_set, 1, vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .add_image(vk::DescriptorImageInfo {
            // The sampler (Telling the pipeline how to sample the texture, including repeat, border, etc.)
            sampler      : texture.sampler,
            // The image's view (images are never directly accessed by the shader, but rather through views defining subresources).
            image_view   : texture.view,
            // The current layout of the image (Note: Should always fit the actual use, e.g. shader read).
            image_layout : texture.layout,
        });

    DescriptorSetsUpdateCI::new()
        .add_write(&ubo_write)
        .add_write(&sampler_write)
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
        .add_attachment(color_attachment)
        .add_attachment(depth_attachment)
        .add_subpass(subpass_description)
        .add_dependency(dependency0)
        .add_dependency(dependency1)
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
        .cull_face(vk::CullModeFlags::NONE, vk::FrontFace::CLOCKWISE);

    let blend_attachment = BlendAttachmentSCI::new();
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

    let vert_module = ShaderModuleCI::new(vert_codes)
        .build(device)?;
    let frag_module = ShaderModuleCI::new(frag_codes).build(device)?;

    // Pipeline Layout.
    let layout = PipelineLayoutCI::new()
        .add_set_layout(set_layout)
        .build(device)?;

    // Pipeline.
    let mut pipeline_ci = GraphicsPipelineCI::new(render_pass, layout);

    let shaders = [
        ShaderStageCI::new(vk::ShaderStageFlags::VERTEX, vert_module),
        ShaderStageCI::new(vk::ShaderStageFlags::FRAGMENT, frag_module),
    ];
    pipeline_ci.set_shaders(&shaders);
    pipeline_ci.set_vertex_input(Vertex::input_description());
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


use ash::vk;

use std::path::Path;

use vkbase::context::{VkDevice, VkSwapchain};
use vkbase::ci::VkObjectBuildableCI;
use vkbase::ci::shader::{ShaderModuleCI, ShaderStageCI};
use vkbase::context::VulkanContext;
use vkbase::utils::color::VkColor;
use vkbase::FrameAction;
use vkbase::VkResult;

use vkexamples::VkExampleBackendRes;
use crate::text::{TextPool, TextInfo, GlyphImages};

const TEXT_VERTEX_SHADER_SOURCE_PATH  : &'static str = "examples/src/text-overlay/text.vert.glsl";
const TEXT_FRAGMENT_SHADER_SOURCE_PATH: &'static str = "examples/src/text-overlay/text.frag.glsl";
const RENDERING_TEXT: &'static str = "Sample Text";

pub struct VulkanExample {

    backend_res: VkExampleBackendRes,

    text_glyphs: GlyphImages,
    text_pool: TextPool,

    pipelines: PipelineStaff,
    descriptors: DescriptorStaff,
}

struct PipelineStaff {
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
}

impl VulkanExample {

    pub fn new(context: &VulkanContext, hidpi_factor: f32) -> VkResult<VulkanExample> {

        let device = &context.device;
        let swapchain = &context.swapchain;

        let render_pass = setup_renderpass(device, &context.swapchain)?;

        let mut backend_res = VkExampleBackendRes::new(device, swapchain, render_pass)?;
        backend_res.enable_depth_attachment(false);

        let text_glyphs = GlyphImages::from_font(device, include_bytes!("../../../assets/fonts/Roboto-Regular.ttf"))?;
        let text_pool = TextPool::new(device, swapchain.dimension, hidpi_factor)?;
        let descriptors = setup_descriptor(device, &text_glyphs)?;

        let pipelines = prepare_pipelines(device, swapchain.dimension, backend_res.render_pass, descriptors.layout)?;

        let target = VulkanExample {
            backend_res, descriptors, pipelines,
            text_glyphs, text_pool,
        };
        Ok(target)
    }
}

impl vkbase::RenderWorkflow for VulkanExample {

    fn init(&mut self, device: &VkDevice) -> VkResult<()> {

        let text = TextInfo {
            content: String::from(RENDERING_TEXT),
            scale  : 24.0,
            color: VkColor::new_u8(128, 0, 128, 255),
            location: vk::Offset2D { x: 0, y: 0 },
        };
        self.text_pool.add_text(text)?;
        self.text_pool.update_texts(device, &self.text_glyphs)?;

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
        device.discard(self.pipelines.pipeline);

        self.text_pool.update_texts(device, &self.text_glyphs)?;

        let render_pass = setup_renderpass(device, new_chain)?;
        self.backend_res.swapchain_reload(device, new_chain, render_pass)?;
        self.pipelines = prepare_pipelines(device, self.backend_res.dimension, self.backend_res.render_pass, self.descriptors.layout)?;

        self.record_commands(device, self.backend_res.dimension)?;

        Ok(())
    }

    fn receive_input(&mut self, inputer: &vkbase::EventController, _delta_time: f32) -> FrameAction {

        if inputer.is_key_active() {

            if inputer.key.is_key_pressed(winit::VirtualKeyCode::Escape) {
                return FrameAction::Terminal
            }
        }

        FrameAction::Rendering
    }

    fn deinit(&mut self, device: &mut VkDevice) -> VkResult<()> {

        self.discard(device);
        Ok(())
    }
}

impl VulkanExample {

    fn record_commands(&self, device: &VkDevice, dimension: vk::Extent2D) -> VkResult<()> {

        let clear_values = [vkexamples::DEFAULT_CLEAR_COLOR.clone()];

        for (i, &command) in self.backend_res.commands.iter().enumerate() {

            use vkbase::command::{VkCmdRecorder, CmdGraphicsApi, IGraphics};
            use vkbase::ci::pipeline::RenderPassBI;

            let recorder: VkCmdRecorder<IGraphics> = VkCmdRecorder::new(&device.logic, command);

            let render_pass_bi = RenderPassBI::new(self.backend_res.render_pass, self.backend_res.framebuffers[i])
                .render_extent(dimension)
                .clear_values(&clear_values);

            recorder.begin_record()?
                .begin_render_pass(render_pass_bi)
                .bind_pipeline(self.pipelines.pipeline)
                .bind_descriptor_sets(self.pipelines.layout, 0, &[self.descriptors.set], &[]);

            self.text_pool.record_command(&recorder);

            recorder
                .end_render_pass()
                .end_record()?;
        }

        Ok(())
    }

    fn discard(&self, device: &mut VkDevice) {

        device.discard(self.descriptors.layout);
        device.discard(self.descriptors.pool);

        device.discard(self.pipelines.pipeline);
        device.discard(self.pipelines.layout);

        self.text_pool.discard(device);
        self.text_glyphs.discard(device);
        self.backend_res.discard(device);
    }
}


struct DescriptorStaff {
    pool   : vk::DescriptorPool,
    set    : vk::DescriptorSet,
    layout : vk::DescriptorSetLayout,
}

fn setup_descriptor(device: &VkDevice, glyphs: &GlyphImages) -> VkResult<DescriptorStaff> {

    use vkbase::ci::descriptor::{DescriptorPoolCI, DescriptorSetLayoutCI};
    use vkbase::ci::descriptor::{DescriptorSetAI, DescriptorImageSetWI, DescriptorSetsUpdateCI};

    // Descriptor Pool.
    let descriptor_pool = DescriptorPoolCI::new(1)
        .add_descriptor(vk::DescriptorType::COMBINED_IMAGE_SAMPLER, 1)
        .build(device)?;

    // `sampled_image_descriptor` represent shader codes as follows:
    // layout (binding = 0) uniform sampler2D font_glyphs;
    let samplers_tmp = [glyphs.text_sampler];
    let sampled_image_descriptor = vk::DescriptorSetLayoutBinding {
        binding: 0,
        descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        descriptor_count: 1,
        stage_flags: vk::ShaderStageFlags::FRAGMENT,
        p_immutable_samplers: samplers_tmp.as_ptr(),
    };

    let set_layout = DescriptorSetLayoutCI::new()
        .add_binding(sampled_image_descriptor)
        .build(device)?;

    // Descriptor set.
    let mut descriptor_sets = DescriptorSetAI::new(descriptor_pool)
        .add_set_layout(set_layout)
        .build(device)?;
    let descriptor_set = descriptor_sets.remove(0);

    // update descriptors.
    let sampled_image_write_info = DescriptorImageSetWI::new(descriptor_set, 0, vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .add_image(vk::DescriptorImageInfo {
            sampler: glyphs.text_sampler,
            image_view: glyphs.glyph_view,
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        });

    DescriptorSetsUpdateCI::new()
        .add_write(sampled_image_write_info.value())
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

    // Only use color attachment.
    let color_attachment = AttachmentDescCI::new(swapchain.backend_format)
        .op(vk::AttachmentLoadOp::CLEAR, vk::AttachmentStoreOp::STORE)
        .layout(vk::ImageLayout::UNDEFINED, vk::ImageLayout::PRESENT_SRC_KHR);

    let subpass_description = SubpassDescCI::new(vk::PipelineBindPoint::GRAPHICS)
        .add_color_attachment(0, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

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
        .add_subpass(subpass_description.value())
        .add_dependency(dependency0.value())
        .add_dependency(dependency1.value())
        .build(device)?;

    Ok(render_pass)
}

fn prepare_pipelines(device: &VkDevice, dimension: vk::Extent2D, render_pass: vk::RenderPass, set_layout: vk::DescriptorSetLayout) -> VkResult<PipelineStaff> {

    use vkbase::ci::pipeline::*;

    let viewport_state = ViewportSCI::new()
        .add_viewport(vk::Viewport {
            x: 0.0, y: 0.0,
            width: dimension.width as f32, height: dimension.height as f32,
            min_depth: 0.0, max_depth: 1.0,
        })
        .add_scissor(vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: dimension,
        });

    let rasterization_state = RasterizationSCI::new()
        .polygon(vk::PolygonMode::FILL)
        .cull_face(vk::CullModeFlags::BACK, vk::FrontFace::COUNTER_CLOCKWISE);

    let blend_attachment = BlendAttachmentSCI::new()
        .blend_enable(true)
        .color(vk::BlendOp::ADD, vk::BlendFactor::SRC_ALPHA, vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .alpha(vk::BlendOp::ADD, vk::BlendFactor::SRC_ALPHA, vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .value();
    let blend_state = ColorBlendSCI::new()
        .add_attachment(blend_attachment);

    // Pipeline Layout.
    let pipeline_layout = PipelineLayoutCI::new()
        .add_set_layout(set_layout)
        .build(device)?;

    // base pipeline.
    let mut pipeline_ci = GraphicsPipelineCI::new(render_pass, pipeline_layout);

    pipeline_ci.set_vertex_input(TextPool::input_descriptions());
    pipeline_ci.set_viewport(viewport_state);
    pipeline_ci.set_rasterization(rasterization_state);
    pipeline_ci.set_color_blend(blend_state);


    let mut shader_compiler = vkbase::utils::shaderc::VkShaderCompiler::new()?;
    let vert_codes = shader_compiler.compile_from_path(Path::new(TEXT_VERTEX_SHADER_SOURCE_PATH), shaderc::ShaderKind::Vertex, "[Vertex Shader]", "main")?;
    let frag_codes = shader_compiler.compile_from_path(Path::new(TEXT_FRAGMENT_SHADER_SOURCE_PATH), shaderc::ShaderKind::Fragment, "[Fragment Shader]", "main")?;

    let vert_module = ShaderModuleCI::from_glsl(vk::ShaderStageFlags::VERTEX, vert_codes)
        .build(device)?;
    let frag_module = ShaderModuleCI::from_glsl(vk::ShaderStageFlags::FRAGMENT, frag_codes)
        .build(device)?;

    pipeline_ci.set_shaders(vec![
        ShaderStageCI::new(vk::ShaderStageFlags::VERTEX, vert_module),
        ShaderStageCI::new(vk::ShaderStageFlags::FRAGMENT, frag_module),
    ]);

    let text_pipeline = device.build(&pipeline_ci)?;

    device.discard(vert_module);
    device.discard(frag_module);

    let result = PipelineStaff {
        pipeline: text_pipeline,
        layout: pipeline_layout,
    };
    Ok(result)
}

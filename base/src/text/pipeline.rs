
use std::path::Path;

use ash::vk;

use crate::context::{VkDevice, VkSwapchain};
use crate::ci::shader::{ShaderModuleCI, ShaderStageCI};
use crate::ci::pipeline::VertexInputSCI;
use crate::ci::VkObjectBuildableCI;
use crate::text::{GlyphImages, CharacterVertex};
use crate::VkResult;


const TEXT_VERTEX_SHADER_SOURCE_PATH  : &'static str = "base/src/text/text.vert.glsl";
const TEXT_FRAGMENT_SHADER_SOURCE_PATH: &'static str = "base/src/text/text.frag.glsl";

pub struct TextPipelineAsset {

    descriptor_pool: vk::DescriptorPool,
    descriptor_set: vk::DescriptorSet,
    descriptor_set_layout: vk::DescriptorSetLayout,

    render_pass: vk::RenderPass,
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
}

impl TextPipelineAsset {

    pub fn new(device: &VkDevice, swapchain: &VkSwapchain, glyphs: &GlyphImages) -> VkResult<TextPipelineAsset> {

        let (desc_pool, desc_set, desc_set_layout) = setup_descriptor(device, glyphs)?;
        let render_pass = setup_renderpass(device, swapchain)?;
        let (pipeline, pipeline_layout) = prepare_pipelines(device, swapchain.dimension, render_pass, desc_set_layout);

        let result = TextPipelineAsset {
            descriptor_pool: desc_pool,
            descriptor_set: desc_set,
            descriptor_set_layout: desc_set_layout,
            render_pass, pipeline, pipeline_layout,
        };
        Ok(result)
    }
}

fn setup_descriptor(device: &VkDevice, glyphs: &GlyphImages) -> VkResult<(vk::DescriptorPool, vk::DescriptorSet, vk::DescriptorSetLayout)> {

    use crate::ci::descriptor::{DescriptorPoolCI, DescriptorSetLayoutCI};
    use crate::ci::descriptor::{DescriptorSetAI, DescriptorImageSetWI, DescriptorSetsUpdateCI};

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

    Ok((descriptor_pool, descriptor_set, set_layout))
}

fn setup_renderpass(device: &VkDevice, swapchain: &VkSwapchain) -> VkResult<vk::RenderPass> {

    use crate::ci::pipeline::RenderPassCI;
    use crate::ci::pipeline::{AttachmentDescCI, SubpassDescCI, SubpassDependencyCI};

    // Don't clear the framebuffer (like the renderpass from the example does)
    let color_attachment = AttachmentDescCI::new(swapchain.backend_format)
        .op(vk::AttachmentLoadOp::LOAD, vk::AttachmentStoreOp::STORE)
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

fn prepare_pipelines(device: &VkDevice, dimension: vk::Extent2D, render_pass: vk::RenderPass, set_layout: vk::DescriptorSetLayout) -> VkResult<(vk::Pipeline, vk::PipelineLayout)> {

    use crate::ci::pipeline::*;

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

    pipeline_ci.set_vertex_input(input_descriptions());
    pipeline_ci.set_viewport(viewport_state);
    pipeline_ci.set_rasterization(rasterization_state);
    pipeline_ci.set_color_blend(blend_state);

    let mut shader_compiler = crate::utils::shaderc::VkShaderCompiler::new()?;
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

    Ok((text_pipeline, pipeline_layout))
}

fn input_descriptions() -> VertexInputSCI {

    VertexInputSCI::new()
        .add_binding(vk::VertexInputBindingDescription {
            binding: 0,
            stride : ::std::mem::size_of::<CharacterVertex>() as _,
            input_rate: vk::VertexInputRate::VERTEX,
        })
        .add_attribute(vk::VertexInputAttributeDescription {
            location: 0,
            binding : 0,
            format  : vk::Format::R32G32_SFLOAT,
            offset  : offset_of!(CharacterVertex, pos) as _,
        })
        .add_attribute(vk::VertexInputAttributeDescription {
            location: 1,
            binding : 0,
            format  : vk::Format::R32G32_SFLOAT,
            offset  : offset_of!(CharacterVertex, uv) as _,
        })
        .add_attribute(vk::VertexInputAttributeDescription {
            location: 2,
            binding : 0,
            format  : vk::Format::R32G32B32A32_SFLOAT,
            offset  : offset_of!(CharacterVertex, color) as _,
        })
}

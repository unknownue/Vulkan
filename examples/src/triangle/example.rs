
use ash::vk;
use vkbase::context::VkDevice;
use vkbase::{VkResult, VkError};
use vkbase::FrameAction;

use std::ptr;

use data::{Vertex, VertexBuffer, IndexBuffer, UniformBuffer, UboVS};

pub struct VulkanExample {

    vertex_buffer: VertexBuffer,

    index_buffer: IndexBuffer,

    uniform_buffer: UniformBuffer,

    /// The pipeline layout is used by a pipeline to access the descriptor sets.
    ///
    /// It defines interface (without binding any actual data) between the shader stages used by the pipeline and the shader resources.
    ///
    /// A pipeline layout can be shared among multiple pipelines as long as their interfaces match.
    pipeline_layout: vk::PipelineLayout,

    /// Pipelines (often called "pipeline state objects") are used to bake all states that affect a pipeline.
    ///
    /// While in OpenGL every state can be changed at (almost) any time, Vulkan requires to layout the graphics (and compute) pipeline states upfront.
    ///
    /// So for each combination of non-dynamic pipeline states you need a new pipeline (there are a few exceptions to this not discussed here).
    ///
    /// Even though this adds a new dimension of planing ahead, it's a great opportunity for performance optimizations by the driver.
    pipeline: vk::Pipeline,

    /// The descriptor set layout describes the shader binding layout (without actually referencing descriptor).
    ///
    /// Like the pipeline layout, it's pretty much a blueprint and can be used with different descriptor sets as long as their layout matches.
    descriptor_set_layout: vk::DescriptorSetLayout,

    /// The descriptor set stores the resources bound to the binding points in a shader.
    ///
    /// It connects the binding points of the different shaders with the buffers and images used for those bindings.
    descriptor_set: vk::DescriptorSet,
}

impl vkbase::Workflow for VulkanExample {

    fn init(&mut self, _device: &VkDevice) -> VkResult<()> {
        Ok(())
    }

    fn render_frame(&mut self, device: &VkDevice, device_available: vk::Fence, image_available: vk::Semaphore, image_index: usize, delta_time: f32) -> VkResult<vk::Semaphore> {

        Ok(image_available)
    }

    fn swapchain_reload(&mut self, _device: &VkDevice) -> VkResult<()> {
        Ok(())
    }

    fn receive_input(&mut self, delta_time: f32) -> FrameAction {

        FrameAction::Rendering
    }

    fn deinit(&mut self, device: &VkDevice) -> VkResult<()> {

        self.discard(device);
        Ok(())
    }
}

impl VulkanExample {

    fn discard(&self, device: &VkDevice) {

        let destructor = &device.logic.handle;
        // clean up used Vulkan resources.
        unsafe {
            destructor.destroy_pipeline(self.pipeline, None);
            destructor.destroy_pipeline_layout(self.pipeline_layout, None);
            destructor.destroy_descriptor_set_layout(self.descriptor_set_layout, None);

            destructor.destroy_buffer(self.vertex_buffer.buffer, None);
            destructor.free_memory(self.vertex_buffer.memory, None);

            destructor.destroy_buffer(self.index_buffer.buffer, None);
            destructor.free_memory(self.index_buffer.memory, None);

            destructor.destroy_buffer(self.uniform_buffer.buffer, None);
            destructor.free_memory(self.uniform_buffer.memory, None);
        }
    }
}


fn setup_descriptor_pool(device: &VkDevice) -> VkResult<vk::DescriptorPool> {

    // We need to tell the API the number of max. requested descriptors per type
    let pool_sizes = vec![
        // This example only uses one descriptor type (uniform buffer) and only requests one descriptor of this type.
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
        },
    ];

    // Create the global descriptor pool
    // All descriptors used in this example are allocated from this pool.
    let descriptor_pool_ci = vk::DescriptorPoolCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::DescriptorPoolCreateFlags::empty(),
        // Set the max number of descriptor sets that can be requested from this pool
        // (requesting beyond this limit will result in an error).
        max_sets: 1,
        pool_size_count: pool_sizes.len() as _,
        p_pool_sizes: pool_sizes.as_ptr(),
    };

    let descriptor_pool = unsafe {
        device.logic.handle.create_descriptor_pool(&descriptor_pool_ci, None)
            .map_err(|_| VkError::create("Descriptor Pool"))?
    };

    Ok(descriptor_pool)
}

fn setup_descriptor_layout(device: &VkDevice) -> VkResult<(vk::DescriptorSetLayout, vk::PipelineLayout)> {

    // Setup layout of descriptors used in this example.
    // Basically connects the different shader stages to descriptors for binding uniform buffers, image samplers, etc.
    // So every shader binding should map to one descriptor set layout binding.

    let layout_bindings = [
        // Binding 0: Uniform buffer (Vertex shader)
        vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::VERTEX,
            p_immutable_samplers: ptr::null(),
        },
    ];

    let descriptor_layout_ci = vk::DescriptorSetLayoutCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: layout_bindings.len() as _,
        p_bindings: layout_bindings.as_ptr(),
    };

    let descriptor_set_layout = unsafe {
        device.logic.handle.create_descriptor_set_layout(&descriptor_layout_ci, None)
            .map_err(|_| VkError::create("Descriptor Set Layout"))?
    };

    let pipeline_layout_ci = vk::PipelineLayoutCreateInfo {
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::PipelineLayoutCreateFlags::empty(),
        set_layout_count: 1,
        p_set_layouts   : &descriptor_set_layout,
        push_constant_range_count: 0,
        p_push_constant_ranges   : ptr::null(),
    };

    let pipeline_layout = unsafe {
        device.logic.handle.create_pipeline_layout(&pipeline_layout_ci, None)
            .map_err(|_| VkError::create("Pipeline Layout"))?
    };

    Ok((descriptor_set_layout, pipeline_layout))
}

fn setup_descriptor_set(device: &VkDevice, pool: vk::DescriptorPool, layout: vk::DescriptorSetLayout, uniforms: &UniformBuffer) -> VkResult<vk::DescriptorSet> {

    // Allocate a new descriptor set from descriptor pool.
    let set_layouts = [layout];

    let descriptor_set_allot_ci = vk::DescriptorSetAllocateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
        p_next: ptr::null(),
        descriptor_pool: pool,
        descriptor_set_count: set_layouts.len() as _,
        p_set_layouts: set_layouts.as_ptr(),
    };

    let descriptor_set = unsafe {
        device.logic.handle.allocate_descriptor_sets(&descriptor_set_allot_ci)
            .map_err(|_| VkError::create("Allocate Descriptor Set"))?
    }.pop().unwrap();


    // Update the descriptor set determining the shader binding points.
    // For every binding point used in a shader there needs to be one descriptor set matching that binding point.
    let buffer_write_info = [uniforms.descriptor.clone()];

    // Binding 0 : Uniform buffer.
    let write_infos = [
        vk::WriteDescriptorSet {
            s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
            p_next: ptr::null(),
            dst_set: descriptor_set,
            // Binds this uniform buffer to binding point 0.
            dst_binding: 0,
            dst_array_element: 0,
            descriptor_count: 1,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            p_image_info: ptr::null(),
            p_buffer_info: buffer_write_info.as_ptr(),
            p_texel_buffer_view: ptr::null(),
        },
    ];

    unsafe {
        device.logic.handle.update_descriptor_sets(&write_infos, &[]);
    }

    Ok(descriptor_set)
}

fn setup_renderpass(device: &VkDevice) -> VkResult<vk::RenderPass> {

    // Render pass setup:
    // Render passes are a new concept in Vulkan.
    // They describe the attachments used during rendering and may contain multiple subpasses with attachment dependencies.
    // This allows the driver to know up-front what the rendering will look like and is a good opportunity to optimize especially on tile-based renderers (with multiple subpasses).
    // Using sub pass dependencies also adds implicit layout transitions for the attachment used, so we don't need to add explicit image memory barriers to transform them.

    // This example will use a single render pass with one subpass.

    // Descriptors for the attachments used by this renderpass.
    let attachments = [
        vk::AttachmentDescription {
            flags: vk::AttachmentDescriptionFlags::empty(),
            // Use the color format selected by the swapchain.
            format: Format
            // Don't use multi sampling in this example.
            samples: vk::SampleCountFlags::TYPE_1,
            // Clear this attachment at the start of the render pass.
            load_op: vk::AttachmentLoadOp::CLEAR,
            // Keep it's contents after the render pass is finished (for displaying it).
            store_op: vk::AttachmentStoreOp::STORE,
            // Don't use stencil, so don't care for load.
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            // Layout at render pass start. Initial doesn't matter, so use undefined here.
            initial_layout: vk::ImageLayout::UNDEFINED,
            // Layout to which the attachment is transitioned when the render pass is finished.
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        },
        vk::AttachmentDescription {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format: DepthFormat,
            samples: vk::SampleCountFlags::TYPE_1,
            // Clear depth at start of first subpass.
            load_op: vk::AttachmentLoadOp::CLEAR,
            // Don't need depth after render pass has finished (DONT_CARE may result in better performance)
            store_op: vk::AttachmentStoreOp::DONT_CARE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            // Transition to depth/stencil attachment.
            final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        },
    ];

    // Setup attachment references.
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

    // Setup a single subpass reference
    let subpass_descs = [
        vk::SubpassDescription {
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
        }
    ];

    // Setup subpass dependencies:
    // These will add the implicit attachment layout transitions specified by the attachment descriptions.
    // The actual usage layout is preserved through the layout specified in the attachment reference.
    // Each subpass dependency will introduce a memory and execution dependency between the source and dest subpass described by srcStageMask, dstStageMask, srcAccessMask, dstAccessMask (and dependencyFlags is set).
    // Note: vk::SUBPASS_EXTERNAL is a special constant that refers to all commands executed outside of the actual renderpass).
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

    let renderpass_ci = vk::RenderPassCreateInfo {
        s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::RenderPassCreateFlags::empty(),
        attachment_count: attachments.len() as _,
        p_attachments   : attachments.as_ptr(),
        subpass_count   : subpass_descs.len() as _,
        p_subpasses     : subpass_descs.as_ptr(),
        dependency_count: dependencies.len() as _,
        p_dependencies  : dependencies.as_ptr(),
    };

    let render_pass = unsafe {
        device.logic.handle.create_render_pass(&renderpass_ci, None)
            .map_err(|_| VkError::create("Render Pass"))?
    };
    Ok(render_pass)
}

fn prepare_pipelines(device: &VkDevice, render_pass: vk::RenderPass) -> VkResult<()> {

    // Create the graphics pipeline used in this example.
    // Vulkan uses the concept of rendering pipelines to encapsulate fixed states, replacing OpenGL's complex state machine.
    // A pipeline is then stored and hashed on the GPU making pipeline changes very fast.
    // Note: There are still a few dynamic states that are not directly part of the pipeline (but the info that they are used is).


    // Construct the different states making up the pipeline

    // Input assembly state describes how primitives are assembled.
    // This pipeline will assemble vertex data as a triangle lists.
    let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
        p_next: ptr::null(),
        flags : vk::PipelineInputAssemblyStateCreateFlags::empty(),
        topology: vk::PrimitiveTopology::TRIANGLE_LIST,
        primitive_restart_enable: vk::FALSE,
    };

    // Rasterization state
    let rasterization_state = vk::PipelineRasterizationStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        p_next: ptr::null(),
        flags : vk::PipelineRasterizationStateCreateFlags::empty(),
        depth_clamp_enable         : vk::FALSE,
        rasterizer_discard_enable  : vk::FALSE,
        polygon_mode               : vk::PolygonMode::FILL,
        cull_mode                  : vk::CullModeFlags::NONE,
        front_face                 : vk::FrontFace::COUNTER_CLOCKWISE,
        depth_bias_enable          : vk::FALSE,
        depth_bias_constant_factor : 0.0,
        depth_bias_clamp           : 0.0,
        depth_bias_slope_factor    : 0.0,
        line_width                 : 1.0,
    };

    // Color blend state describes how blend factors are calculated (if used).
    // Need one blend attachment state per color attachment (even if blending is not used).
    let blend_attachments = [
        vk::PipelineColorBlendAttachmentState {
            blend_enable: vk::FALSE,
            src_color_blend_factor: vk::BlendFactor::ONE,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B | vk::ColorComponentFlags::A,
        },
    ];
    let blend_state = vk::PipelineColorBlendStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        p_next: ptr::null(),
        flags : vk::PipelineColorBlendStateCreateFlags::empty(),
        logic_op_enable: vk::FALSE,
        logic_op       : vk::LogicOp::COPY,
        attachment_count: blend_attachments.len() as _,
        p_attachments   : blend_attachments.as_ptr(),
        blend_constants : [0.0; 4]
    };

    // Viewport state sets the number of viewports and scissor used in this pipeline.
    // Note: This is actually overridden by the dynamic states.
    let viewport_state = vk::PipelineViewportStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        p_next: ptr::null(),
        flags : vk::PipelineViewportStateCreateFlags::empty(),
        viewport_count : 1,
        p_viewports    : ptr::null(),
        scissor_count  : 1,
        p_scissors     : ptr::null(),
    };


    // Depth and stencil state containing depth and stencil compare and test operations.
    // Here only use depth tests and want depth tests and writes to be enabled and compare with less or equal.
    let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
        p_next: ptr::null(),
        flags : vk::PipelineDepthStencilStateCreateFlags::empty(),
        depth_test_enable        : vk::TRUE,
        depth_write_enable       : vk::TRUE,
        depth_compare_op         : vk::CompareOp::LESS_OR_EQUAL,
        depth_bounds_test_enable : vk::FALSE,
        stencil_test_enable      : vk::FALSE,
        front: vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op   : vk::CompareOp::ALWAYS,
            compare_mask : 0,
            write_mask   : 0,
            reference    : 0,
        },
        back: vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op   : vk::CompareOp::ALWAYS,
            compare_mask : 0,
            write_mask   : 0,
            reference    : 0,
        },
        min_depth_bounds: 0.0,
        max_depth_bounds: 1.0,
    };

    // Multi sampling state
    // the state must still be set and passed to the pipeline if disable.
    let multisample_state = vk::PipelineMultisampleStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        p_next: ptr::null(),
        flags : vk::PipelineMultisampleStateCreateFlags::empty(),
        rasterization_samples: vk::SampleCountFlags::TYPE_1,
        sample_shading_enable: vk::FALSE,
        min_sample_shading: 0.0,
        p_sample_mask: ptr::null(),
        alpha_to_coverage_enable: vk::FALSE,
        alpha_to_one_enable     : vk::FALSE,
    };

    // Enable dynamic states
    // Most states are baked into the pipeline, but there are still a few dynamic states that can be changed within a command buffer.
    // To be able to change these we need do specify which dynamic states will be changed using this pipeline. Their actual states are set later on in the command buffer.
    // This example will set the viewport and scissor using dynamic states.
    let dynamics = [
        vk::DynamicState::VIEWPORT,
        vk::DynamicState::SCISSOR,
    ];
    let dynamic_state = vk::PipelineDynamicStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
        p_next: ptr::null(),
        flags : vk::PipelineDynamicStateCreateFlags::empty(),
        dynamic_state_count: dynamics.len() as _,
        p_dynamic_states   : dynamics.as_ptr(),
    };


    // Vertex input descriptions
    // Specifies the vertex input parameters for a pipeline
    let input_descriptions = Vertex::input_description();


//    let pipeline_ci = vk::GraphicsPipelineCreateInfo {
//        s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
//        p_next: ptr::null(),
//        flags: vk::PipelineCreateFlags::empty(),
//        stage_count: u32
//        p_stages: *const PipelineShaderStageCreateInfo
//        p_vertex_input_state: *const PipelineVertexInputStateCreateInfo
//        p_input_assembly_state: *const PipelineInputAssemblyStateCreateInfo
//        p_tessellation_state: *const PipelineTessellationStateCreateInfo
//        p_viewport_state: *const PipelineViewportStateCreateInfo
//        p_rasterization_state: *const PipelineRasterizationStateCreateInfo
//        p_multisample_state: *const PipelineMultisampleStateCreateInfo
//        p_depth_stencil_state: *const PipelineDepthStencilStateCreateInfo
//        p_color_blend_state: *const PipelineColorBlendStateCreateInfo
//        p_dynamic_state: *const PipelineDynamicStateCreateInfo
//        layout: PipelineLayout
//        render_pass: RenderPass
//        subpass: u32
//        base_pipeline_handle: Pipeline
//        base_pipeline_index: i32
//    };

    unimplemented!()
}

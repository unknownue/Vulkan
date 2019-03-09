
use ash::vk;

use std::ptr;
use std::mem;
use std::path::Path;

use arrayvec::ArrayVec;

use vkbase::context::{VkDevice, VkSwapchain};
use vkbase::ci::VkObjectBuildableCI;
use vkbase::ci::buffer::BufferCI;
use vkbase::ci::vma::{VmaBuffer, VmaAllocationCI};
use vkbase::gltf::VkglTFModel;
use vkbase::texture::Texture2D;
use vkbase::context::VulkanContext;
use vkbase::{FlightCamera, FrameAction};
use vkbase::{vkbytes, vkptr, Point3F, Matrix4F, Vector3F};
use vkbase::{VkResult, VkErrorKind};

use vkexamples::VkExampleBackend;

const VERTEX_SHADER_SOURCE_PATH  : &'static str = "examples/src/descriptorsets/cube.vert.glsl";
const FRAGMENT_SHADER_SOURCE_PATH: &'static str = "examples/src/descriptorsets/cube.frag.glsl";

const CUBE_MODEL_PATH: &'static str = "assets/models/cube.gltf";
const CUBE_COUNT: usize = 2;
const CUBE_TEXTURE_PATHS: [&'static str; CUBE_COUNT] = [
    "assets/textures/crate01_color_height_rgba.ktx",
    "assets/textures/crate02_color_height_rgba.ktx",
];

// TODO: Check box to toggle animation is not yet implemented.
const IS_ANIMATE: bool = true;


pub struct VulkanExample {

    backend: VkExampleBackend,

    model: VkglTFModel,

    cubes: ArrayVec<[Cube; 2]>,

    pipelines: PipelineStaff,
    descriptors: DescriptorStaff,

    camera: FlightCamera,

    is_toggle_event: bool,
}

struct PipelineStaff {
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
}

impl VulkanExample {

    pub fn new(context: &mut VulkanContext) -> VkResult<VulkanExample> {

        let device = &mut context.device;
        let swapchain = &context.swapchain;
        let dimension = swapchain.dimension;

        let mut camera = FlightCamera::new()
            .place_at(Point3F::new(0.0, 0.0, 5.0))
            .view_distance(0.1, 512.0)
            .screen_aspect_ratio(dimension.width as f32 / dimension.height as f32)
            .build();
        camera.set_move_speed(5.0);


        let render_pass = setup_renderpass(device, &context.swapchain)?;
        let backend = VkExampleBackend::new(device, swapchain, render_pass)?;

        let model = prepare_model(device)?;

        let mut cubes = prepare_uniform(device, &camera)?;
        let descriptors = setup_descriptor(device, &mut cubes, &model)?;

        let pipelines = prepare_pipelines(device, &model, backend.render_pass, descriptors.layout)?;

        let target = VulkanExample {
            backend, model, cubes, descriptors, pipelines, camera,
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

        for cube in self.cubes.into_iter() {
            device.vma_discard(cube.uniform_buffer)?;
            cube.texture.discard_by(device)?;
        }
        device.vma_discard(self.model)?;
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
                .set_scissor(0, &[scissor])
                .bind_pipeline(self.pipelines.pipeline);

            // Render cubes with separate descriptor sets.
            for j in 0..CUBE_COUNT {

                let render_params = vkbase::gltf::ModelRenderParams {
                    descriptor_set : self.cubes[j].descriptor_set,
                    pipeline_layout: self.pipelines.layout,
                    material_stage : None,
                };

                self.model.record_command(&recorder, &render_params);
            }

            self.backend.ui_renderer.record_command(&recorder);

            recorder
                .end_render_pass()
                .end_record()?;
        }

        Ok(())
    }

    fn update_uniforms(&mut self, delta_time: f32) -> VkResult<()> {

        if IS_ANIMATE || self.is_toggle_event {

            let model_translation: [Matrix4F; 2] = [
                Matrix4F::new_translation(&Vector3F::new(-2.0, 0.0, 0.0)),
                Matrix4F::new_translation(&Vector3F::new(1.5, 0.5, 0.0)),
            ];

            self.cubes[0].rotation.x = (self.cubes[0].rotation.x + 2.5 * delta_time) % 360.0;
            self.cubes[0].matrices.model = model_translation[0] * Matrix4F::new_rotation(self.cubes[0].rotation);

            self.cubes[1].rotation.y = (self.cubes[1].rotation.y + 2.0 * delta_time) % 360.0;
            self.cubes[1].matrices.model = model_translation[1] * Matrix4F::new_rotation(self.cubes[1].rotation);

            self.cubes[0].matrices.view = self.camera.view_matrix();
            self.cubes[1].matrices.view = self.camera.view_matrix();

            use vkbase::utils::memory::copy_to_ptr;
            copy_to_ptr(self.cubes[0].uniform_buffer.info.get_mapped_data() as vkptr, &[self.cubes[0].matrices]);
            copy_to_ptr(self.cubes[1].uniform_buffer.info.get_mapped_data() as vkptr, &[self.cubes[1].matrices]);
        }

        Ok(())
    }
}

// Prepare model from glTF file.
pub fn prepare_model(device: &mut VkDevice) -> VkResult<VkglTFModel> {

    use vkbase::gltf::{GltfModelInfo, load_gltf};
    use vkbase::gltf::{AttributeFlags, NodeAttachmentFlags};

    let model_info = GltfModelInfo {
        path: Path::new(CUBE_MODEL_PATH),
        // specify model's vertices layout.
        // in cube.vert.glsl:
        // layout (location = 0) in vec3 inPos;
        // layout (location = 1) in vec2 inUV;
        attribute: AttributeFlags::POSITION | AttributeFlags::TEXCOORD_0,
        // specify model's node attachment layout.
        // in cube.vert.glsl
        // layout (set = 0, binding = 1) uniform DynNode {
        //     mat4 transform;
        // } dyn_node;
        node: NodeAttachmentFlags::TRANSFORM_MATRIX,
    };

    let model = load_gltf(device, model_info)?;
    Ok(model)
}


/// The uniform structure for each descriptor set.
///
/// layout (set = 0, binding = 0) uniform UBOMatrices {
///     mat4 projection;
///     mat4 view;
///     mat4 model;
/// } ubo;
#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct UBOMatrices {
    projection: Matrix4F,
    view      : Matrix4F,
    model     : Matrix4F,
}

struct Cube {
    matrices: UBOMatrices,
    descriptor_set: vk::DescriptorSet,
    uniform_buffer: VmaBuffer,
    texture : Texture2D,
    rotation: Vector3F,
}


fn prepare_uniform(device: &mut VkDevice, camera: &FlightCamera) -> VkResult<ArrayVec<[Cube; 2]>> {

    let mut cubes = ArrayVec::new();

    for i in 0..CUBE_COUNT {

        let ubo_buffer = {
            let uniform_ci = BufferCI::new(mem::size_of::<UBOMatrices>() as vkbytes)
                .usage(vk::BufferUsageFlags::UNIFORM_BUFFER);
            let allocation_ci = VmaAllocationCI::new(vma::MemoryUsage::CpuOnly, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)
                .flags(vma::AllocationCreateFlags::MAPPED);
            let uniform_allocation = device.vma.create_buffer(&uniform_ci.value(), allocation_ci.as_ref())
                .map_err(VkErrorKind::Vma)?;

            VmaBuffer::from(uniform_allocation)
        };

        let cube = Cube {
            matrices: UBOMatrices {
                projection: camera.proj_matrix(),
                model     : Matrix4F::identity(),
                view      : camera.view_matrix(),
            },
            // the descriptor_set member will be set in setup_descriptor() method.
            descriptor_set: vk::DescriptorSet::null(),
            uniform_buffer: ubo_buffer,
            texture : Texture2D::load(device, Path::new(CUBE_TEXTURE_PATHS[i]), vk::Format::R8G8B8A8_UNORM)?,
            rotation: Vector3F::new(0.0, 0.0, 0.0),
        };
        cubes.push(cube);
    }

    Ok(cubes)
}

struct DescriptorStaff {
    pool   : vk::DescriptorPool,
    layout : vk::DescriptorSetLayout,
}


/*
    SaschaWillems's comment:

    Descriptor set layout

    The layout describes the shader bindings and types used for a certain descriptor layout and as such must match the shader bindings

    Shader bindings used in this example:

    VS:
        layout (set = 0, binding = 0) uniform UBOMatrices ...
        layout (set = 0, binding = 1) uniform DynNode ...

    FS :
        layout (set = 0, binding = 2) uniform sampler2D ...;

*/
fn setup_descriptor(device: &VkDevice, cubes: &mut ArrayVec<[Cube; 2]>, model: &VkglTFModel) -> VkResult<DescriptorStaff> {

    use vkbase::ci::descriptor::{DescriptorPoolCI, DescriptorSetLayoutCI};
    use vkbase::ci::descriptor::{DescriptorSetAI, DescriptorBufferSetWI, DescriptorImageSetWI, DescriptorSetsUpdateCI};

    /*
        SaschaWillems's comment:

        Descriptor pool

        Actual descriptors are allocated from a descriptor pool telling the driver what types and how many
        descriptors this application will use.

        An application can have multiple pools (e.g. for multiple threads) with any number of descriptor types
        as long as device limits are not surpassed.

        It's good practice to allocate pools with actually required descriptor types and counts.

    */
    // Descriptor Pool.
    // Max. number of descriptor sets that can be allocated from this pool (one per object).
    let descriptor_pool = DescriptorPoolCI::new(CUBE_COUNT as _)
        // Uniform buffers: 1 per object.
        .add_descriptor(vk::DescriptorType::UNIFORM_BUFFER, CUBE_COUNT as _)
        // Dynamic uniform buffers: 1 per object.
        .add_descriptor(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC, CUBE_COUNT as _)
        // Combined image samples : 1 per mesh texture(in the example, 1 mesh per object).
        .add_descriptor(vk::DescriptorType::COMBINED_IMAGE_SAMPLER, CUBE_COUNT as _)
        .build(device)?;

    /*
        Binding 0: Uniform buffer (used to pass matrices matrices).
        in cube.vert.glsl:

        layout (set = 0, binding = 0) uniform UBOMatrices {
           mat4 projection;
           mat4 view;
           mat4 model;
        } ubo;
    */
    let ubo_descriptor = vk::DescriptorSetLayoutBinding {
        // Shader binding point.
        binding: 0,
        // The type of descriptor to bind.
        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
        // Binding contains one element (can be used for array bindings).
        descriptor_count: 1,
        // Accessible from the vertex shader only (flags can be combined to make it accessible to multiple shader stages).
        stage_flags: vk::ShaderStageFlags::VERTEX,
        p_immutable_samplers: ptr::null(),
    };

    /*
        Binding 1: Dynamic uniform buffer(used for matrix properties in glTF Node hierarchy).
        in cube.vert.glsl:

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
        Binding 2: Combined Image sampler (used to pass per object texture information).
        in cube.frag.glsl:

        layout (set = 0, binding = 2) uniform sampler2D samplerColorMap;
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

    /*
        SaschaWillems's comment:

        Descriptor sets

        Using the shared descriptor set layout and the descriptor pool we will now allocate the descriptor sets.

        Descriptor sets contain the actual descriptor fo the objects (buffers, images) used at render time.

    */

    for i in 0..CUBE_COUNT {

        let mut descriptor_sets = DescriptorSetAI::new(descriptor_pool)
            .add_set_layout(set_layout)
            .build(device)?;
        cubes[i].descriptor_set = descriptor_sets.remove(0);

        // Update the descriptor set with the actual descriptors matching shader bindings set in the layout.

        // Binding 0: Object matrices Uniform buffer.
        let ubo_write_info = DescriptorBufferSetWI::new(cubes[i].descriptor_set, 0, vk::DescriptorType::UNIFORM_BUFFER)
            .add_buffer(vk::DescriptorBufferInfo {
                buffer: cubes[i].uniform_buffer.handle,
                offset: 0,
                range : mem::size_of::<UBOMatrices>() as vkbytes,
            });
        // Binding 1: Node hierarchy transform matrix in glTF.
        let node_write_info = DescriptorBufferSetWI::new(cubes[i].descriptor_set, 1, vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
            .add_buffer(model.nodes.node_descriptor());
        // Binding 2: Object texture.
        let sampler_write_info = DescriptorImageSetWI::new(cubes[i].descriptor_set, 2, vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .add_image(cubes[i].texture.descriptor);

        /*
            SaschaWillems's comment:

            Execute the writes to update descriptors for this set.

            Note that it's also possible to gather all writes and only run updates once, even for multiple sets.

            This is possible because each VkWriteDescriptorSet also contains the destination set to be updated.

            For simplicity we will update once per set instead.
        */

        DescriptorSetsUpdateCI::new()
            .add_write(ubo_write_info.value())
            .add_write(node_write_info.value())
            .add_write(sampler_write_info.value())
            .update(device);
    }


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

    // Pipeline Layout.
    // The pipeline layout is based on the descriptor set layout we created above.
    let layout = PipelineLayoutCI::new()
        .add_set_layout(set_layout)
        .build(device)?;

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

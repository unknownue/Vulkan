
use ash::vk;
use ash::version::DeviceV1_0;

use vkbase::context::{VkDevice, VkSwapchain};
use vkbase::ci::VkObjectBuildableCI;
use vkbase::ci::sync::SemaphoreCI;
use vkbase::ui::UIRenderer;
use vkbase::vkuint;
use vkbase::{VkResult, VkError};

pub const DEFAULT_CLEAR_COLOR: vk::ClearValue = vk::ClearValue {
    color: vk::ClearColorValue {
        float32: [0.025, 0.025, 0.025, 1.0]
    }
};


pub struct VkExampleBackendRes {

    pub dimension: vk::Extent2D,
    pub render_pass: vk::RenderPass,
    pub framebuffers: Vec<vk::Framebuffer>,

    pub await_rendering: vk::Semaphore,

    pub command_pool: vk::CommandPool,
    /// render command buffer for each framebuffer.
    pub commands: Vec<vk::CommandBuffer>,

    ui_renderer: UIRenderer,

    depth_image: DepthImage,
    is_use_depth_attachment: bool,
}

struct DepthImage {
    image: vk::Image,
    view : vk::ImageView,
    memory: vk::DeviceMemory,
}

impl VkExampleBackendRes {

    pub fn new(device: &VkDevice, swapchain: &VkSwapchain, renderpass: vk::RenderPass) -> VkResult<VkExampleBackendRes> {

        let dimension = swapchain.dimension;
        let (command_pool, commands) = setup_commands(device, swapchain.frame_in_flight as _)?;
        let depth_image = setup_depth_image(device, swapchain.dimension)?;
        let await_rendering = device.build(&SemaphoreCI::new())?;

        // TODO: Fix dpi_factor.
        let ui_renderer = UIRenderer::new(device, swapchain, command_pool, renderpass, 1.0)?;

        let mut target = VkExampleBackendRes {
            depth_image, await_rendering, ui_renderer,
            commands, command_pool, dimension,
            render_pass: renderpass,
            framebuffers: Vec::new(),
            is_use_depth_attachment: true,
        };
        target.setup_framebuffers(device, swapchain)?;

        Ok(target)
    }

    pub fn enable_depth_attachment(&mut self, is_enable: bool) {
        self.is_use_depth_attachment = is_enable;
    }

    fn setup_framebuffers(&mut self, device: &VkDevice, swapchain: &VkSwapchain) -> VkResult<()> {

        use vkbase::ci::pipeline::FramebufferCI;

        // create a frame buffer for every image in the swapchain.
        self.framebuffers = Vec::with_capacity(swapchain.frame_in_flight());

        for i in 0..swapchain.frame_in_flight() {

            let mut framebuffer_ci = FramebufferCI::new_2d(self.render_pass, self.dimension)
                .add_attachment(swapchain.images[i].view); // color attachment is the view of the swapchain image.

            if self.is_use_depth_attachment {
                framebuffer_ci = framebuffer_ci.add_attachment(self.depth_image.view);
            }

            let framebuffer = framebuffer_ci.build(device)?;
            self.framebuffers.push(framebuffer);
        }

        Ok(())
    }

    pub fn swapchain_reload(&mut self, device: &VkDevice, new_chain: &VkSwapchain, render_pass: vk::RenderPass) -> VkResult<()> {

        self.dimension = new_chain.dimension;

        device.discard(self.depth_image.view);
        device.discard(self.depth_image.image);
        device.discard(self.depth_image.memory);
        self.depth_image = setup_depth_image(device, self.dimension)?;

        device.discard(&self.framebuffers);
        self.render_pass = render_pass;
        self.setup_framebuffers(device, new_chain)?;

        unsafe {
            device.logic.handle.reset_command_pool(self.command_pool, vk::CommandPoolResetFlags::empty())
                .map_err(|_| VkError::device("Reset Command Pool"))?;
        }

        Ok(())
    }

    pub fn discard(&self, device: &VkDevice) {

        self.ui_renderer.discard(device);

        device.discard(self.render_pass);
        device.discard(&self.framebuffers);

        device.discard(self.command_pool);

        device.discard(self.depth_image.view);
        device.discard(self.depth_image.image);
        device.discard(self.depth_image.memory);

        device.discard(self.await_rendering);
    }
}

fn setup_depth_image(device: &VkDevice, dimension: vk::Extent2D) -> VkResult<DepthImage> {

    use vkbase::ci::image::{ImageCI, ImageViewCI};
    use vkbase::ci::memory::MemoryAI;
    use vkbase::utils::memory::get_memory_type_index;

    let (image, image_requirement) = ImageCI::new_2d(device.phy.depth_format, dimension)
        .usages(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
        .build(device)?;

    let memory_index = get_memory_type_index(device, image_requirement.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL);
    let memory = MemoryAI::new(image_requirement.size, memory_index)
        .build(device)?;

    // bind depth image to memory.
    device.bind_memory(image, memory, 0)?;

    let view = ImageViewCI::new(image, vk::ImageViewType::TYPE_2D, device.phy.depth_format)
        .aspect_mask(vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL)
        .build(device)?;

    let result = DepthImage { image, view, memory };
    Ok(result)
}

fn setup_commands(device: &VkDevice, buffer_count: vkuint) -> VkResult<(vk::CommandPool, Vec<vk::CommandBuffer>)> {

    use vkbase::ci::command::{CommandPoolCI, CommandBufferAI};

    let command_pool = CommandPoolCI::new(device.logic.queues.graphics.family_index)
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .build(device)?;

    let command_buffers = CommandBufferAI::new(command_pool, buffer_count)
        .build(device)?;

    Ok((command_pool, command_buffers))
}

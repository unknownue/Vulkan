
use ash::vk;
use ash::version::DeviceV1_0;

use lazy_static::lazy_static;

use vkbase::ci::VkObjectBuildableCI;
use vkbase::ci::sync::SemaphoreCI;
use vkbase::ci::image::{ImageCI, ImageViewCI};
use vkbase::ci::vma::{VmaImage, VmaAllocationCI};
use vkbase::ui::{UIRenderer, TextInfo, TextID, TextType, TextHAlign};

use vkbase::context::{VkDevice, VkSwapchain};
use vkbase::utils::color::VkColor;
use vkbase::vkuint;
use vkbase::{VkResult, VkError, VkErrorKind};

lazy_static! {

    pub static ref DEFAULT_CLEAR_VALUES: Vec<vk::ClearValue> = vec![
        vk::ClearValue { color: vk::ClearColorValue { float32: [0.025, 0.025, 0.025, 1.0] } },
        vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 } },
    ];

}

pub struct VkExampleBackend {

    pub dimension: vk::Extent2D,
    pub render_pass: vk::RenderPass,
    pub framebuffers: Vec<vk::Framebuffer>,

    pub await_rendering: vk::Semaphore,

    pub command_pool: vk::CommandPool,
    /// render command buffer for each framebuffer.
    pub commands: Vec<vk::CommandBuffer>,

    pub ui_renderer: UIRenderer,
    fps_text_id: Option<TextID>,

    depth_image: DepthImage,
    is_use_depth_attachment: bool,
}

struct DepthImage {
    image: VmaImage,
    view : vk::ImageView,
}

impl VkExampleBackend {

    pub fn new(device: &mut VkDevice, swapchain: &VkSwapchain, renderpass: vk::RenderPass) -> VkResult<VkExampleBackend> {

        let dimension = swapchain.dimension;
        let (command_pool, commands) = setup_commands(device, swapchain.frame_in_flight as _)?;
        let depth_image = setup_depth_image(device, swapchain.dimension)?;
        let await_rendering = device.build(&SemaphoreCI::new())?;

        let ui_renderer = UIRenderer::new(device, swapchain, renderpass)?;

        let mut target = VkExampleBackend {
            depth_image, await_rendering, ui_renderer,
            commands, command_pool, dimension,
            fps_text_id: None,
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

    pub fn swapchain_reload(&mut self, device: &mut VkDevice, new_chain: &VkSwapchain, render_pass: vk::RenderPass) -> VkResult<()> {

        self.dimension = new_chain.dimension;
        self.ui_renderer.swapchain_reload(device, new_chain, render_pass)?;

        let mut new_depth_image = setup_depth_image(device, self.dimension)?;
        std::mem::swap(&mut new_depth_image, &mut self.depth_image);

        device.discard(new_depth_image.view);
        device.vma_discard(new_depth_image.image)?;

        device.discard(&self.framebuffers);
        device.discard(self.render_pass);

        self.render_pass = render_pass;
        self.setup_framebuffers(device, new_chain)?;

        unsafe {
            device.logic.handle.reset_command_pool(self.command_pool, vk::CommandPoolResetFlags::empty())
                .map_err(|_| VkError::device("Reset Command Pool"))?;
        }
        Ok(())
    }

    pub fn set_basic_ui(&mut self, device: &VkDevice, title: &str) -> VkResult<()> {

        let title_text = TextInfo {
            content: String::from(title),
            scale: 12.0,
            align: TextHAlign::Left,
            color: VkColor::WHITE,
            location: vk::Offset2D { x: 5, y: 0 },
            r#type: TextType::Static,
        };

        let device_text = TextInfo {
            content: device.phy.device_name.clone(),
            scale: 12.0,
            align: TextHAlign::Left,
            color: VkColor::WHITE,
            location: vk::Offset2D { x: 5, y: 40 },
            r#type: TextType::Static,
        };

        let fps_text = TextInfo {
            content: String::from("FPS: 00.00"),
            scale: 12.0,
            align: TextHAlign::Left,
            color: VkColor::WHITE,
            location: vk::Offset2D { x: 5, y: 80 },
            r#type: TextType::Dynamic { capacity: 15 },
        };

        self.ui_renderer.add_text(title_text)?;
        self.ui_renderer.add_text(device_text)?;
        self.fps_text_id = Some(self.ui_renderer.add_text(fps_text)?);

        Ok(())
    }

    pub fn update_fps_text(&mut self, inputer: &vkbase::EventController) {

        // update text on fps per second.
        if inputer.fps_counter.is_tick_second() {

            if let Some(text_id) = self.fps_text_id {
                let fps = format!("FPS: {}", inputer.fps_counter.fps());
                self.ui_renderer.change_text(fps, text_id);
            }
        }
    }

    pub fn discard_by(self, device: &mut VkDevice) -> VkResult<()> {

        self.ui_renderer.discard_by(device)?;

        device.discard(self.render_pass);
        device.discard(&self.framebuffers);

        device.discard(self.command_pool);

        device.discard(self.depth_image.view);
        device.vma_discard(self.depth_image.image)?;

        device.discard(self.await_rendering);

        Ok(())
    }
}

fn setup_depth_image(device: &mut VkDevice, dimension: vk::Extent2D) -> VkResult<DepthImage> {

    let image = {
        let depth_ci = ImageCI::new_2d(device.phy.depth_format, dimension)
            .usages(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT);
        let allocation_ci = VmaAllocationCI::new(vma::MemoryUsage::GpuOnly, vk::MemoryPropertyFlags::DEVICE_LOCAL);
        let depth_allocation = device.vma.create_image(
            &depth_ci, &allocation_ci)
            .map_err(VkErrorKind::Vma)?;
        VmaImage::from(depth_allocation)
    };

    let view = ImageViewCI::new(image.handle, vk::ImageViewType::TYPE_2D, device.phy.depth_format)
        .sub_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL,
            base_mip_level: 0,
            level_count   : 1,
            base_array_layer: 0,
            layer_count     : 1,
        }).build(device)?;

    let result = DepthImage { image, view };
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

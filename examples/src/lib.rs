
use ash::vk;
use ash::version::DeviceV1_0;

use vkbase::context::{VkDevice, VkSwapchain};
use vkbase::ci::VkObjectBuildableCI;
use vkbase::ci::sync::SemaphoreCI;
use vkbase::vkuint;
use vkbase::{VkResult, VkError};


pub struct VkExampleBackendRes {

    depth_image: DepthImage,

    pub dimension: vk::Extent2D,
    pub framebuffers: Vec<vk::Framebuffer>,

    pub await_rendering: vk::Semaphore,

    pub command_pool: vk::CommandPool,
    pub commands: Vec<vk::CommandBuffer>,
}

struct DepthImage {
    image: vk::Image,
    view : vk::ImageView,
    memory: vk::DeviceMemory,
}

impl VkExampleBackendRes {

    pub fn new(device: &VkDevice, swapchain: &VkSwapchain) -> VkResult<VkExampleBackendRes> {

        let dimension = swapchain.dimension;
        let (command_pool, commands) = setup_commands(device, swapchain.frame_in_flight as _)?;
        let depth_image = setup_depth_image(device, swapchain.dimension)?;
        let await_rendering = device.build(&SemaphoreCI::new())?;

        let target = VkExampleBackendRes {
            depth_image, await_rendering,
            commands, command_pool, dimension,
            framebuffers: Vec::new(),
        };
        Ok(target)
    }

    pub fn setup_framebuffers(&mut self, device: &VkDevice, swapchain: &VkSwapchain, render_pass: vk::RenderPass) -> VkResult<()> {

        if self.framebuffers.is_empty() == false {
            self.framebuffers.clear();
        }

        use vkbase::ci::pipeline::FramebufferCI;

        // create a frame buffer for every image in the swapchain.
        self.framebuffers = Vec::with_capacity(swapchain.frame_in_flight());
        let dimension = swapchain.dimension.clone();

        for i in 0..swapchain.frame_in_flight() {

            let framebuffer = FramebufferCI::new_2d(render_pass, dimension)
                .add_attachment(swapchain.images[i].view) // color attachment is the view of the swapchain image.
                .add_attachment(self.depth_image.view) // depth/stencil attachment is the same for all frame buffers.
                .build(device)?;
            self.framebuffers.push(framebuffer);
        }

        Ok(())
    }

    pub fn swapchain_reload(&mut self, device: &VkDevice, new_chain: &VkSwapchain, render_pass: vk::RenderPass) -> VkResult<()> {

        self.dimension =new_chain.dimension;

        device.discard(self.depth_image.view);
        device.discard(self.depth_image.image);
        device.discard(self.depth_image.memory);
        self.depth_image = setup_depth_image(device, self.dimension)?;

        device.discard(&self.framebuffers);
        self.setup_framebuffers(device, new_chain, render_pass)?;

        unsafe {
            device.logic.handle.reset_command_pool(self.command_pool, vk::CommandPoolResetFlags::empty())
                .map_err(|_| VkError::device("Reset Command Pool"))?;
        }

        Ok(())
    }

    pub fn discard(&self, device: &VkDevice) {

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

    let (image, image_requirement) = ImageCI::new_2d(device.phy.depth_format, dimension)
        .usages(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
        .build(device)?;

    let memory_index = get_memory_type_index(device, image_requirement.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL);
    let memory = MemoryAI::new(image_requirement.size, memory_index)
        .build(device)?;

    unsafe {
        device.logic.handle.bind_image_memory(image, memory, 0)
            .map_err(|_| VkError::device("Bind Image Memory."))?;
    }

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

pub fn get_memory_type_index(device: &VkDevice, mut type_bits: vkuint, properties: vk::MemoryPropertyFlags) -> vkuint {

    // Iterate over all memory types available for the device used in this example.
    let memories = &device.phy.memories;
    for i in 0..memories.memory_type_count {
        if (type_bits & 1) == 1 {
            if memories.memory_types[i as usize].property_flags.contains(properties) {
                return i
            }
        }

        type_bits >>= 1;
    }

    panic!("Could not find a suitable memory type")
}


mod pipeline;
mod text;


use ash::vk;

use crate::context::{VkDevice, VkSwapchain};
use crate::command::{VkCmdRecorder, IGraphics, CmdGraphicsApi};
use crate::VkResult;

use crate::ui::text::TextPool;
use crate::ui::pipeline::UIPipelineAsset;


pub struct UIRenderer {

    /// the vulkan resource to render text.
    pipeline_asset: UIPipelineAsset,

    text_pool: TextPool,
}

impl UIRenderer {

    pub fn new(device: &VkDevice, swapchain: &VkSwapchain, command_pool: vk::CommandPool, renderpass: vk::RenderPass, dpi_factor: f32) -> VkResult<UIRenderer> {

        let text_pool = TextPool::new(device, swapchain.dimension, dpi_factor)?;
        let pipeline_asset = pipeline::UIPipelineAsset::new(device, swapchain, command_pool, renderpass, text_pool.glyphs_ref())?;

        let renderer = UIRenderer { pipeline_asset, text_pool };
        Ok(renderer)
    }

    pub fn record_command(&self, recorder: &VkCmdRecorder<IGraphics>) {

        recorder.bind_pipeline(self.pipeline_asset.pipeline)
            .bind_descriptor_sets(self.pipeline_asset.pipeline_layout, 0, &[self.pipeline_asset.descriptor_set], &[]);

        self.text_pool.record_command(recorder);
    }

    pub fn discard(&self, device: &VkDevice) {

        self.pipeline_asset.discard(device);
        self.text_pool.discard(device);
    }
}

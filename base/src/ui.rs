
pub use self::text::{TextInfo, TextHAlign};

mod pipeline;
mod text;


use ash::vk;

use crate::context::{VkDevice, VkSwapchain};
use crate::command::{VkCmdRecorder, IGraphics, CmdGraphicsApi};
use crate::ui::pipeline::UIPipelineAsset;
use crate::ui::text::TextPool;
use crate::VkResult;



pub struct UIRenderer {

    /// the vulkan resource to render text.
    pipeline_asset: UIPipelineAsset,

    text_pool: TextPool,
}

impl UIRenderer {

    pub fn new(device: &VkDevice, swapchain: &VkSwapchain, renderpass: vk::RenderPass, dpi_factor: f32) -> VkResult<UIRenderer> {

        let text_pool = TextPool::new(device, swapchain.dimension, dpi_factor)?;
        let pipeline_asset = pipeline::UIPipelineAsset::new(device, swapchain, renderpass, text_pool.glyphs_ref())?;

        let renderer = UIRenderer { pipeline_asset, text_pool };
        Ok(renderer)
    }

    pub fn record_command(&self, recorder: &VkCmdRecorder<IGraphics>) {

        recorder.bind_pipeline(self.pipeline_asset.pipeline)
            .bind_descriptor_sets(self.pipeline_asset.pipeline_layout, 0, &[self.pipeline_asset.descriptor_set], &[]);

        self.text_pool.record_command(recorder);
    }

    pub fn swapchain_reload(&mut self, device: &VkDevice, new_chain: &VkSwapchain, renderpass: vk::RenderPass) -> VkResult<()> {

        self.pipeline_asset.swapchain_reload(device, new_chain, renderpass)?;
        self.text_pool.swapchain_reload()
    }

    pub fn add_text(&mut self, text: TextInfo) -> VkResult<()> {
        self.text_pool.add_text(text)
    }

    pub fn discard(&self, device: &VkDevice) {

        self.pipeline_asset.discard(device);
        self.text_pool.discard(device);
    }
}

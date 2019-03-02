
pub use self::text::{TextInfo, TextID, TextType, TextHAlign};

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

    pub fn new(device: &mut VkDevice, swapchain: &VkSwapchain, renderpass: vk::RenderPass) -> VkResult<UIRenderer> {

        let text_pool = TextPool::new(device, swapchain.dimension)?;
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
        self.text_pool.swapchain_reload();

        Ok(())
    }

    pub fn add_text(&mut self, text: TextInfo) -> VkResult<TextID> {
        self.text_pool.add_text(text)
    }

    pub fn change_text(&mut self, content: String, update_text: TextID) {
        self.text_pool.change_text(content, update_text);
    }

    pub fn discard(&self, device: &mut VkDevice) -> VkResult<()> {

        self.pipeline_asset.discard(device);
        self.text_pool.discard(device)
    }
}

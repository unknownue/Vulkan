
use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::{VulkanContext, VkDevice, SwapchainSyncError};
use crate::workflow::RenderWorkflow;
use crate::workflow::window::WindowContext;
use crate::input::EventController;
use crate::utils::time::VkTimeDuration;
use crate::utils::frame::{FrameCounter, FrameAction};
use crate::error::{VkResult, VkError};


pub struct ProcPipeline {

    window: WindowContext,
    vulkan: VulkanContext,

    syncs: SyncResource,

    frame_counter: FrameCounter,
}

impl ProcPipeline {

    pub fn new(window: WindowContext, vulkan: VulkanContext) -> VkResult<ProcPipeline> {

        let frame_in_flight = vulkan.swapchain.frame_in_flight();
        let syncs = SyncResource::new(&vulkan.device, frame_in_flight)?;
        let frame_counter = FrameCounter::new(frame_in_flight);

        let target = ProcPipeline { window, vulkan, syncs, frame_counter };
        Ok(target)
    }

    pub fn frame_in_flight(&self) -> usize {
        self.vulkan.swapchain.frame_in_flight()
    }

    pub fn launch(mut self, mut app: impl RenderWorkflow) -> VkResult<()> {

        app.init(&self.vulkan.device)?;

        self.main_loop(&mut app)?;

        self.vulkan.wait_idle()?;
        app.deinit(&mut self.vulkan.device)?;
        // free the program specific resource.
        drop(app);
        // and then free vulkan context resource.
        self.syncs.discard(&self.vulkan.device);
        self.vulkan.discard();

        Ok(())
    }

    fn main_loop(&mut self, app: &mut impl RenderWorkflow) -> VkResult<()> {

        let mut event_handler = EventController::default();

        'loop_marker: loop {

            macro_rules! response_feedback {
                ($action:ident) => {
                    match $action {
                        | FrameAction::Rendering => {},
                        | FrameAction::SwapchainRecreate => {

                            self.vulkan.wait_idle()?;
                            self.vulkan.recreate_swapchain(&self.window)?;
                            app.swapchain_reload(&mut self.vulkan.device, &self.vulkan.swapchain)?;
                        },
                        | FrameAction::Terminal => {
                            break 'loop_marker
                        },
                    }
                }
            }

            let delta_time = event_handler.fps_counter.delta_time();

            self.window.event_loop.poll_events(|event| {
                event_handler.record_event(event);
            });
            let window_feedback = event_handler.current_action();
            response_feedback!(window_feedback);

            let input_feedback = app.receive_input(&event_handler, delta_time);
            response_feedback!(input_feedback);

            let render_feedback = self.render_frame(app, delta_time)?;
            response_feedback!(render_feedback);

            event_handler.tick_frame();
            self.frame_counter.tick_frame();
        }

        Ok(())
    }

    fn render_frame(&mut self, app: &mut impl RenderWorkflow, delta_time: f32) -> VkResult<FrameAction> {

        // wait and acquire next image. -------------------------------------
        let fence_ready = self.syncs.sync_fences[self.frame_counter.current_frame()];
        unsafe {
            self.vulkan.device.logic.handle.wait_for_fences(&[fence_ready], true, VkTimeDuration::Infinite.into())
                .map_err(|_| VkError::device("Fence waiting"))?;
        }

        let acquire_image_index = match self.vulkan.swapchain.next_image(Some(self.syncs.await_present), None) {
            | Ok(image_index) => image_index,
            | Err(e) => match e {
                | SwapchainSyncError::SurfaceOutDate
                | SwapchainSyncError::SubOptimal => {
                    return Ok(FrameAction::SwapchainRecreate)
                },
                | SwapchainSyncError::TimeOut
                | SwapchainSyncError::Unknown => {
                    return Err(VkError::custom(e.to_string()))
                },
            }
        };

        unsafe {
            self.vulkan.device.logic.handle.reset_fences(&[fence_ready])
                .map_err(|_| VkError::device("Fence Resetting"))?;
        }
        // ------------------------------------------------------------------

        // call command buffer(activate pipeline to draw) -------------------
        let await_render = app.render_frame(&self.vulkan.device, fence_ready, self.syncs.await_present, acquire_image_index as _, delta_time)?;
        // ------------------------------------------------------------------

        // present image. ---------------------------------------------------
        // TODO: Add ownership transfer if need.
        // see https://github.com/KhronosGroup/Vulkan-Docs/wiki/Synchronization-Examples.
        // or see https://software.intel.com/en-us/articles/api-without-secrets-introduction-to-vulkan-part-3#inpage-nav-6-3
        match self.vulkan.swapchain.present(&[await_render], acquire_image_index) {
            | Ok(_) => {},
            | Err(e) => match e {
                | SwapchainSyncError::SurfaceOutDate
                | SwapchainSyncError::SubOptimal => {
                    return Ok(FrameAction::SwapchainRecreate)
                },
                | SwapchainSyncError::TimeOut
                | SwapchainSyncError::Unknown => {
                    return Err(VkError::custom(e.to_string()))
                },
            },
        }
        // ------------------------------------------------------------------

        Ok(FrameAction::Rendering)
    }
}



struct SyncResource {

    frame_count: usize,

    await_present: vk::Semaphore,
    sync_fences : Vec<vk::Fence>,
}

impl SyncResource {

    pub fn new(device: &VkDevice, frame_count: usize) -> VkResult<SyncResource> {

        use crate::ci::sync::{SemaphoreCI, FenceCI};

        let await_present = device.build(&SemaphoreCI::new())?;

        let mut sync_fences = Vec::with_capacity(frame_count);
        let fence_ci = FenceCI::new(true);

        for _ in 0..frame_count {
            sync_fences.push(device.build(&fence_ci)?);
        }

        let syncs = SyncResource { frame_count, await_present, sync_fences };
        Ok(syncs)
    }

    #[allow(dead_code)]
    fn reset(&mut self, device: &VkDevice) -> VkResult<()> {

        self.discard(device);
        *self = SyncResource::new(device, self.frame_count)?;

        Ok(())
    }

    fn discard(&mut self, device: &VkDevice) {

        device.discard(self.await_present);
        device.discard(&self.sync_fences);

        self.sync_fences.clear();
    }
}

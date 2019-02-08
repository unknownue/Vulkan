
use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::{VulkanContext, VkDevice, SwapchainSyncError};
use crate::workflow::Workflow;
use crate::workflow::window::WindowContext;
use crate::utils::fps::FpsCounter;
use crate::utils::time::VkTimeDuration;
use crate::utils::frame::{FrameCounter, FrameAction};
use crate::error::{VkResult, VkError};

use std::ptr;

pub struct ProcPipeline {

    window: WindowContext,
    vulkan: VulkanContext,

    syncs: SyncResource,

    frame_counter: FrameCounter,
    fps_counter: FpsCounter,
}

impl ProcPipeline {

    pub fn new(window: WindowContext, vulkan: VulkanContext) -> VkResult<ProcPipeline> {

        let frame_in_flight = vulkan.swapchain.frame_in_flight();
        let syncs = SyncResource::new(&vulkan.device, frame_in_flight)?;
        let frame_counter = FrameCounter::new(frame_in_flight);
        let fps_counter = FpsCounter::new();

        let target = ProcPipeline { window, vulkan, syncs, frame_counter, fps_counter };
        Ok(target)
    }

    pub fn launch(&mut self, mut app: impl Workflow) -> VkResult<()> {

        app.init(&self.vulkan.device)?;

        self.main_loop(&mut app)?;

        app.deinit(&self.vulkan.device)?;
        self.vulkan.wait_idle()?;
        // free the program specific resource.
        drop(app);
        // and then free vulkan context resource.
        self.syncs.discard(&self.vulkan.device);
        self.vulkan.discard();

        Ok(())
    }

    fn main_loop(&mut self, app: &mut impl Workflow) -> VkResult<()> {

        'loop_marker: loop {

            let delta_time = self.fps_counter.delta_time();

            self.window.event_loop.poll_events(|_event| {
                // record event here.
                // ...
            });

            app.receive_input(delta_time);
            self.render_frame(app, delta_time)?;

            match self.frame_counter.current_action() {
                | FrameAction::Rendering => {},
                | FrameAction::SwapchainRecreate => {

                    self.vulkan.wait_idle()?;
                    self.vulkan.recreate_swapchain(&self.window.handle)?;
                    app.swapchain_reload(&self.vulkan.device)?;
                },
                | FrameAction::Terminal => {
                    break  'loop_marker
                },
            }

            self.frame_counter.next_frame();
            self.fps_counter.tick_frame();
        }

        Ok(())
    }

    fn render_frame(&mut self, app: &mut impl Workflow, delta_time: f32) -> VkResult<()> {

        // wait and acquire next image. -------------------------------------
        let fence_ready = self.syncs.sync_fences[self.frame_counter.current_frame()];
        unsafe {
            self.vulkan.device.logic.handle.wait_for_fences(&[fence_ready], true, VkTimeDuration::Infinite.into())
                .map_err(|_| VkError::device("Fence waiting"))?;
        }

        let image_to_acquire = self.syncs.image_awaits[self.frame_counter.current_frame()];
        let acquire_image_index = match self.vulkan.swapchain.next_image(Some(image_to_acquire), None) {
            | Ok(image_index) => image_index,
            | Err(e) => match e {
                | SwapchainSyncError::SurfaceOutDate
                | SwapchainSyncError::SubOptimal => {
                    self.frame_counter.set_action(FrameAction::SwapchainRecreate);
                    return Ok(())
                },
                | SwapchainSyncError::TimeOut
                | SwapchainSyncError::Unknown => {
                    return Err(VkError::other(e.to_string()))
                },
            }
        };

        unsafe {
            self.vulkan.device.logic.handle.reset_fences(&[fence_ready])
                .map_err(|_| VkError::device("Fence Resetting"))?;
        }
        // ------------------------------------------------------------------

        // call command buffer(activate pipeline to draw) -------------------
        let image_ready_to_present = app.render_frame(&self.vulkan.device, fence_ready, image_to_acquire, acquire_image_index as _, delta_time)?;
        // ------------------------------------------------------------------

        // present image. ---------------------------------------------------
        // TODO: Add ownership transfer if need.
        // see https://github.com/KhronosGroup/Vulkan-Docs/wiki/Synchronization-Examples.
        // or see https://software.intel.com/en-us/articles/api-without-secrets-introduction-to-vulkan-part-3#inpage-nav-6-3
        self.vulkan.swapchain.present(&[image_ready_to_present], acquire_image_index)
            .or_else(|e| match e {
                | SwapchainSyncError::SurfaceOutDate
                | SwapchainSyncError::SubOptimal => {
                    self.frame_counter.set_action(FrameAction::SwapchainRecreate);
                    Ok(())
                },
                | SwapchainSyncError::TimeOut
                | SwapchainSyncError::Unknown => {
                    Err(VkError::other(e.to_string()))
                },
            })
        // ------------------------------------------------------------------
    }
}



struct SyncResource {

    frame_count: usize,

    image_awaits: Vec<vk::Semaphore>,
    sync_fences : Vec<vk::Fence>,
}

impl SyncResource {

    pub fn new(device: &VkDevice, frame_count: usize) -> VkResult<SyncResource> {

        let mut image_awaits = Vec::with_capacity(frame_count);
        let mut sync_fences = Vec::with_capacity(frame_count);

        let semaphore_ci = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            p_next: ptr::null(),
            // flags is reserved for future use in API version 1.1.82.
            flags: vk::SemaphoreCreateFlags::empty(),
        };

        let fence_ci = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::FenceCreateFlags::SIGNALED,
        };

        for _ in 0..frame_count {

            unsafe {
                let semaphore = device.logic.handle.create_semaphore(&semaphore_ci, None)
                    .or(Err(VkError::create("Semaphore")))?;
                image_awaits.push(semaphore);

                let fence = device.logic.handle.create_fence(&fence_ci, None)
                    .or(Err(VkError::create("Fence")))?;
                sync_fences.push(fence);
            }
        }

        let syncs = SyncResource { frame_count, image_awaits, sync_fences };
        Ok(syncs)
    }

    fn reset(&mut self, device: &VkDevice) -> VkResult<()> {

        self.discard(device);
        *self = SyncResource::new(device, self.frame_count)?;

        Ok(())
    }

    fn discard(&mut self, device: &VkDevice) {

        unsafe {
            for &semaphore in self.image_awaits.iter() {
                device.logic.handle.destroy_semaphore(semaphore, None);
            }

            for &fence in self.sync_fences.iter() {
                device.logic.handle.destroy_fence(fence, None);
            }
        }

        self.image_awaits.clear();
        self.sync_fences.clear();
    }
}

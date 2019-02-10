
pub struct FrameCounter {

    frame_in_flight: usize,
    current: usize,
}

impl FrameCounter {

    pub fn new(frame_in_flight: usize) -> FrameCounter {

        FrameCounter {
            frame_in_flight,
            current: 0,
        }
    }

    #[inline]
    pub fn current_frame(&self) -> usize {
        self.current.clone()
    }

    #[inline]
    pub fn next_frame(&mut self) {

        self.current = (self.current + 1) % self.frame_in_flight;
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FrameAction {
    /// ordinary action.
    Rendering,
    /// tell program the swapchain has to update to adapt current window surface.
    SwapchainRecreate,
    /// Indicate the program to terminal.
    Terminal,
}

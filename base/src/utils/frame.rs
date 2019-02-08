
pub struct FrameCounter {

    frame_in_flight: usize,
    current: usize,
    action : FrameAction,
}

impl FrameCounter {

    pub fn new(frame_in_flight: usize) -> FrameCounter {

        FrameCounter {
            frame_in_flight,
            current: 0,
            action: FrameAction::Rendering,
        }
    }

    #[inline]
    pub fn current_frame(&self) -> usize {
        self.current.clone()
    }

    #[inline]
    pub fn current_action(&self) -> FrameAction {
        self.action.clone()
    }

    pub fn set_action(&mut self, action: FrameAction) {

        // Only cover current action with higher priority.
        // Rendering should have lower priority than other actions.
        if self.action == FrameAction::Rendering {
            self.action = action;
        }
    }

    #[inline]
    pub fn next_frame(&mut self) {

        self.current = (self.current + 1) % self.frame_in_flight;
        self.action = FrameAction::Rendering;
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

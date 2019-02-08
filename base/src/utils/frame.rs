
pub struct FrameCounter {

    frame_in_flight: usize,
    current: usize,
    action : FrameAction,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FrameAction {
    Rendering,
    SwapchainRecreate,
    Terminal,
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

    #[inline]
    pub fn set_action(&mut self, action: FrameAction) {
        self.action = action;
    }

    #[inline]
    pub fn next_frame(&mut self) {

        self.current = (self.current + 1) % self.frame_in_flight;
        self.action = FrameAction::Rendering;
    }
}

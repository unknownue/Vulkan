
use std::time::Duration;

use crate::vklint;

#[derive(Debug, Copy, Clone)]
pub enum VkTimeDuration {
    Immediate,
    Time(Duration),
    Infinite,
}

impl From<VkTimeDuration> for vklint {

    fn from(time: VkTimeDuration) -> vklint {
        match time {
            | VkTimeDuration::Immediate => 0,
            | VkTimeDuration::Time(time) =>
                (time.subsec_nanos() as vklint) + time.as_secs() * 1_000_000_000,
            | VkTimeDuration::Infinite => vklint::max_value(),
        }
    }
}

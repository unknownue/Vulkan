
use std::time::Instant;

pub const FPS_SAMPLE_COUNT: usize = 5;
pub const FPS_SAMPLE_COUNT_FLOAT: f32 = FPS_SAMPLE_COUNT as f32;
pub const DEFAULT_PREFER_FPS: f32 = 60.0;

pub struct FpsCounter {

    counter: Instant,
    frame_time_prefer: u32, // unit microseconds

    samples: [u32; FPS_SAMPLE_COUNT],
    current_frame: usize,
    delta_frame: u32,
}

impl FpsCounter {

    pub fn new() -> FpsCounter {

        FpsCounter {
            counter: Instant::now(),
            frame_time_prefer: (1000_000.0_f32 / DEFAULT_PREFER_FPS) as u32,
            samples: [0; FPS_SAMPLE_COUNT],
            current_frame: 0,
            delta_frame: 0,
        }
    }

    #[allow(dead_code)]
    pub fn set_prefer_fps(&mut self, prefer_fps: f32) {
        self.frame_time_prefer = (1000_000.0_f32 / prefer_fps) as u32;
    }

    /// Call this function in game loop to update its inner status.
    pub fn tick_frame(&mut self) {
        let time_elapsed = self.counter.elapsed();
        self.counter = Instant::now();

        self.delta_frame = time_elapsed.subsec_micros();
        self.samples[self.current_frame] = self.delta_frame;
        self.current_frame = (self.current_frame + 1) % FPS_SAMPLE_COUNT;
    }

//    TODO: this function seems not work.
//    pub fn keep_fps(&self) {
//
//        use std::thread;
//        use std::Duration;
//        if self.frame_time_prefer > self.delta_frame {
//            let delay = Duration::from_micros((self.frame_time_prefer - self.delta_frame) as u64);
//
//            thread::sleep(delay);
//        }
//    }

    /// Calculate the current FPS.
    #[allow(dead_code)]
    pub fn fps(&self) -> f32 {
        let mut sum = 0_u32;
        self.samples.iter().for_each(|val| {
            sum += val;
        });

        1000_000.0_f32 / (sum as f32 / FPS_SAMPLE_COUNT_FLOAT)
    }

    /// Return current delta time in seconds
    /// this function ignore its second part, since the second is mostly zero.
    pub fn delta_time(&self) -> f32 {
        self.delta_frame as f32 / 1000_000.0_f32 // time in second
    }
}

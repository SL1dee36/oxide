use std::time::Instant;

/// System time tracker providing delta time, total elapsed time, and FPS calculation.
pub struct Time {
    start_time: Instant,
    last_frame_time: Instant,
    delta_time: f32,
    elapsed_time: f32,
    frame_count: u64,
    fps: f32,
    fps_counter: u32,
    fps_timer: f32,
}

impl Time {
    /// Creates a new time tracker initialized to current clock instant.
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_frame_time: now,
            delta_time: 0.0,
            elapsed_time: 0.0,
            frame_count: 0,
            fps: 0.0,
            fps_counter: 0,
            fps_timer: 0.0,
        }
    }

    /// Advances frame timing state, recalculates delta time (clamped to 0.1s), and updates FPS accumulator.
    pub fn update(&mut self) {
        let now = Instant::now();
        let duration = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;

        self.delta_time = duration.as_secs_f32().min(0.1);
        self.elapsed_time = now.duration_since(self.start_time).as_secs_f32();
        self.frame_count += 1;

        self.fps_counter += 1;
        self.fps_timer += self.delta_time;
        if self.fps_timer >= 1.0 {
            self.fps = self.fps_counter as f32 / self.fps_timer;
            self.fps_counter = 0;
            self.fps_timer = 0.0;
        }
    }

    /// Returns time elapsed during the preceding frame in seconds ($\Delta t$).
    #[inline]
    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }

    /// Returns total runtime elapsed since engine startup in seconds.
    #[inline]
    pub fn elapsed(&self) -> f32 {
        self.elapsed_time
    }

    /// Returns total count of frames processed since startup.
    #[inline]
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Returns measured frames per second averaged over the last 1.0s window.
    #[inline]
    pub fn fps(&self) -> f32 {
        self.fps
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}

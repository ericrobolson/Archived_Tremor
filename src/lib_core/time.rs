use time::Instant;
const MILLISECONDS_IN_SECOND: u64 = 1000;

pub type Duration = time::Duration;

pub struct Clock {
    start: Instant,
}

impl Clock {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        return Instant::now() - self.start;
    }
}

pub struct Timer {
    frame_duration: time::Duration,
    hz: u32,
    last_execution: Instant,
}

impl Timer {
    /// Create a new Timer that runs at the specified HZ
    pub fn new(hz: u32) -> Self {
        let mut timer = Self {
            hz: hz,
            frame_duration: Duration::milliseconds(1),
            last_execution: Instant::now(),
        };

        timer.set_hz(hz);

        timer
    }

    pub fn set_hz(&mut self, hz: u32) {
        let mut hz = hz;
        if hz == 0 {
            hz = 1;
        }

        let frame_duration = Duration::milliseconds(MILLISECONDS_IN_SECOND as i64 / hz as i64);
        self.frame_duration = frame_duration;
        self.hz = hz;
    }

    pub fn hz(&self) -> u32 {
        self.hz
    }

    /// Returns the delta in seconds. Only used for gfx interpolation.
    pub fn delta_ratio(&self) -> f32 {
        let now = Instant::now();

        return ((now - self.last_execution) / self.frame_duration) as f32;
    }

    /// Check if the timer can run. If so, update it so that it uses the current instant as the last time ran.
    pub fn can_run(&mut self) -> bool {
        let now = Instant::now();
        let run_game_sim = self.frame_duration <= (now - self.last_execution);

        if run_game_sim {
            self.last_execution = now;
        }

        return run_game_sim;
    }
}

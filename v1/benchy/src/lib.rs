use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::time::{Duration, Instant};

/// Starts a new timer.
pub fn time(function: &'static str) -> Timer {
    Timer::start(function)
}

pub struct Timer {
    start: Instant,
    function: &'static str,
}

impl Timer {
    pub fn start(function: &'static str) -> Self {
        Self {
            function,
            start: Instant::now(),
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let duration = Instant::now() - self.start;
        println!("BENCHY: '{:?}' resulted in {:?}.", self.function, duration);
    }
}

struct BenchyResults {
    pub function: &'static str,
    pub duration: Duration,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

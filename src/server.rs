use crate::event_queue;
use event_queue::*;

pub struct Server {}

impl Server {
    pub fn new() -> Self {
        Self {}
    }

    pub fn execute(&mut self, event_queue: &EventQueue) -> Result<(), String> {
        Ok(())
    }
}

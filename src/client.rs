use crate::event_queue;
use event_queue::*;

use crate::gfx;
use gfx::OpenGlRenderer;

pub struct Client {}

impl Client {
    pub fn new() -> Self {
        Self {}
    }

    pub fn execute(&mut self, event_queue: &EventQueue) -> Result<(), String> {
        // Do gfx stuff?
        Ok(())
    }
}

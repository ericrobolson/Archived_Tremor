use crate::event_queue;
use event_queue::*;

use crate::gfx;
use gfx::OpenGlRenderer;

use crate::lib_core::ecs;
use ecs::World;

pub struct Client {
    world: World,
}

impl Client {
    pub fn new() -> Self {
        Self {
            world: World::new(),
        }
    }

    pub fn execute(&mut self, event_queue: &EventQueue) -> Result<(), String> {
        self.world.dispatch()?;
        // Do gfx stuff?
        Ok(())
    }
}

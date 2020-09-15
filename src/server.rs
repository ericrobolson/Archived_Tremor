use crate::event_queue;
use event_queue::*;

use crate::lib_core::ecs;
use ecs::World;

pub struct Server {
    world: World,
}

impl Server {
    pub fn new() -> Self {
        Self {
            world: World::new(),
        }
    }

    pub fn execute(&mut self, event_queue: &EventQueue) -> Result<(), String> {
        self.world.dispatch()?;
        Ok(())
    }
}

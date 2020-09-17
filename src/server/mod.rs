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

    pub fn execute(
        &mut self,
        event_queue: &EventQueue,
        socket_out_queue: &mut EventQueue,
    ) -> Result<(), String> {
        // TODO: queue up messages to send to clients? Primarily things within 'scope' of a player or ghosts
        self.world.dispatch()?;
        Ok(())
    }
}

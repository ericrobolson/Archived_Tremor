use crate::constants;
use crate::event_queue;
use event_queue::*;

use crate::lib_core::ecs;
use ecs::World;

use crate::network;
use network::connection_layer::ConnectionManager;

pub struct Server {
    world: World,
    connection: ConnectionManager,
}

impl Server {
    pub fn new() -> Self {
        Self {
            world: World::new(constants::SIMULATION_HZ as u32),
            connection: ConnectionManager::new(constants::MAX_SERVER_CONNECTIONS),
        }
    }

    pub fn execute(
        &mut self,
        event_queue: &mut EventQueue,
        socket_out_queue: &mut EventQueue,
    ) -> Result<(), String> {
        self.connection.read_all(event_queue)?;

        // TODO: queue up messages to send to clients? Primarily things within 'scope' of a player or ghosts
        self.world.dispatch()?;

        self.connection.write_all(event_queue, socket_out_queue)?;

        Ok(())
    }
}

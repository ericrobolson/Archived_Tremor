use crate::constants;
use crate::event_queue;
use event_queue::*;

use crate::window;
use window::WindowRenderer;

use crate::lib_core;
use lib_core::{ecs::World, input::ClientInputMapper};

use crate::network;
use network::connection_layer::ConnectionManager;

pub struct Client {
    world: World,
    connection: ConnectionManager,
    input_handler: ClientInputMapper,
}

impl Client {
    pub fn new() -> Self {
        Self {
            world: World::new(constants::SIMULATION_HZ as u32),
            connection: ConnectionManager::new(constants::MAX_CLIENT_CONNECTIONS),
            input_handler: ClientInputMapper::new(constants::SIMULATION_HZ as u32),
        }
    }

    pub fn execute(
        &mut self,
        event_queue: &mut EventQueue,
        socket_out_queue: &mut EventQueue,
        window_renderer: &mut WindowRenderer,
    ) -> Result<(), String> {
        // Connection manager stuff
        self.connection.read_all(event_queue)?;

        // Handle input
        {
            self.input_handler.execute(event_queue)?;
        }

        // Execute sim
        self.world.dispatch()?;

        // Send out events
        self.connection.write_all(event_queue, socket_out_queue)?;
        // Do gfx stuff in here
        window_renderer.render(&self.world);
        Ok(())
    }
}

pub struct RollbackManager {}

impl RollbackManager {
    fn save_state(&mut self) {
        unimplemented!()
    }

    fn load_state(&mut self) {
        unimplemented!()
    }

    fn advance_game_state(&mut self) {
        unimplemented!()
    }

    fn register_input(&mut self, player_id: u8, frame: u16) {
        unimplemented!()
    }
}

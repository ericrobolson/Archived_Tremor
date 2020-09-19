use crate::constants;
use crate::event_queue;
use event_queue::*;

use crate::gfx;
use gfx::OpenGlRenderer;

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
    ) -> Result<(), String> {
        // Connection manager stuff
        self.connection.read_all(event_queue)?;

        // Handle input
        {
            self.input_handler.execute(event_queue)?;

            // While it's not optimal to queue up the input a frame 'late' to send to the server, it does keep all client-specific code contained here. May optimize later.
            for i in 0..event_queue.count() {
                match event_queue.events()[i] {
                    Some((_, e)) => match e {
                        Events::InputPoll(_) => {
                            socket_out_queue.add(e)?;
                        }
                        _ => {}
                    },
                    None => {
                        break;
                    }
                }
            }
        }

        // Execute sim
        self.world.dispatch()?;

        // Send out events
        self.connection.write_all(event_queue, socket_out_queue)?;
        // TODO: Do gfx stuff in here
        Ok(())
    }
}

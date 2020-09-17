use crate::event_queue;
use event_queue::*;

use crate::gfx;
use gfx::OpenGlRenderer;

use crate::lib_core;
use lib_core::{ecs::World, time::Timer};

const SIM_HZ: u32 = 60;

pub struct Client {
    world: World,
}

impl Client {
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
        // While it's not optimal to queue up the input a frame 'late' to send to the server, it does keep all client-specific code contained here.
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

        self.world.dispatch()?;
        // Do gfx stuff?
        Ok(())
    }
}

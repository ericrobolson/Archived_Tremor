use crate::lib_core;
use lib_core::{ecs::World, time::Timer};

use crate::event_queue;
use event_queue::*;

use crate::constants;
use constants::SIMULATION_HZ;

/// Player input struct
pub struct PlayerInput {
    pitch_radians: f32,
    yaw_radians: f32,
    roll_radians: f32,

    action1: bool,
    action2: bool,
    action3: bool,
    action4: bool,
    action5: bool,
    action6: bool,
    action7: bool,
    action8: bool,
    action9: bool,
    action10: bool,
}

struct ClientInputMapper {
    timer: Timer,
    input_state: PlayerInput,
}

impl ClientInputMapper {
    pub fn new() -> Self {
        Self {
            timer: Timer::new(SIMULATION_HZ),
            input_state: PlayerInput {
                pitch_radians: 0.0,
                yaw_radians: 0.0,
                roll_radians: 0.0,

                action1: false,
                action2: false,
                action3: false,
                action4: false,
                action5: false,
                action6: false,
                action7: false,
                action8: false,
                action9: false,
                action10: false,
            },
        }
    }

    pub fn execute(&mut self, event_queue: &mut EventQueue) {
        // Go thru events and update the state.

        // Can directly pass through the mouse look to the GFX api for camera rotation as well while using polling for actual sims

        if self.timer.can_run() {
            // Add new input event to the queue with the current state
        }
    }
}

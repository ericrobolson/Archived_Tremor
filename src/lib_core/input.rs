use crate::lib_core;
use lib_core::{serialization, time::Timer};

use crate::event_queue;
use event_queue::*;

/// Player input struct
#[derive(Copy, Clone, Debug)]
pub struct PlayerInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl PlayerInput {
    pub fn new() -> Self {
        Self {
            up: false,
            down: false,
            left: false,
            right: false,
        }
    }

    pub fn to_bytes(&self) {
        unimplemented!()
    }
}

struct PlayerInputMapper {
    up_keycodes: [Option<u32>; 4],
    down_keycodes: [Option<u32>; 4],
    left_keycodes: [Option<u32>; 4],
    right_keycodes: [Option<u32>; 4],
}
impl PlayerInputMapper {
    fn is_up(keycode: u32) -> bool {
        17 == keycode
    }

    fn is_down(keycode: u32) -> bool {
        31 == keycode
    }

    fn is_left(keycode: u32) -> bool {
        30 == keycode
    }

    fn is_right(keycode: u32) -> bool {
        32 == keycode
    }
}

pub struct ClientInputMapper {
    timer: Timer,
    input_state: PlayerInput,
}

impl ClientInputMapper {
    pub fn new(poll_hz: u32) -> Self {
        Self {
            timer: Timer::new(poll_hz),
            input_state: PlayerInput::new(),
        }
    }

    pub fn execute(&mut self, event_queue: &mut EventQueue) -> Result<(), String> {
        //TODO: load keybindings from a config

        // Go thru events and update the state.
        for i in 0..event_queue.count() {
            match event_queue.events()[i] {
                Some((_duration, e)) => {
                    match e {
                        Events::Keyboard { scancode, pressed } => {
                            let pressed = match pressed {
                                ButtonState::Pressed => true,
                                ButtonState::Released => false,
                            };
                            //TODO: wire up custom bindings

                            if PlayerInputMapper::is_up(scancode) {
                                self.input_state.up = pressed;
                            }

                            if PlayerInputMapper::is_down(scancode) {
                                self.input_state.down = pressed;
                            }

                            if PlayerInputMapper::is_right(scancode) {
                                self.input_state.right = pressed;
                            }

                            if PlayerInputMapper::is_left(scancode) {
                                self.input_state.left = pressed;
                            }
                        }
                        _ => {}
                    }
                }
                None => {
                    break;
                }
            }
        }

        if self.timer.can_run() {
            // Add new input event to the queue with the current state
            event_queue.add(Events::InputPoll(self.input_state))?;
        }

        // Can directly pass through the mouse look to the GFX api for camera rotation as well while using polling for actual sims
        {
            /*
            event_queue.add(Events::GfxView {
                pitch_radians: self.input_state.pitch_radians,
                yaw_radians: self.input_state.yaw_radians,
                roll_radians: self.input_state.roll_radians,
            })?;
            */
        }

        Ok(())
    }
}

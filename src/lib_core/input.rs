use crate::lib_core;
use lib_core::{serialization, time::Timer};

use crate::event_queue;
use event_queue::*;

/// Player input struct
#[derive(Copy, Clone, Debug)]
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
    action11: bool,
    action12: bool,
    action13: bool,
    action14: bool,
    action15: bool,
    action16: bool,
}

impl PlayerInput {
    pub fn to_bytes(&self) {
        let pitch = serialization::serialize_f32(self.pitch_radians);
        let yaw = serialization::serialize_f32(self.yaw_radians);
        let roll = serialization::serialize_f32(self.roll_radians);

        let mut actions: u32 = serialization::new_bit_array();
        actions = serialization::push_bit(actions, self.action1);
        actions = serialization::push_bit(actions, self.action2);
        actions = serialization::push_bit(actions, self.action3);
        actions = serialization::push_bit(actions, self.action4);
        actions = serialization::push_bit(actions, self.action5);
        actions = serialization::push_bit(actions, self.action6);
        actions = serialization::push_bit(actions, self.action7);
        actions = serialization::push_bit(actions, self.action8);
        actions = serialization::push_bit(actions, self.action9);
        actions = serialization::push_bit(actions, self.action10);
        actions = serialization::push_bit(actions, self.action11);
        actions = serialization::push_bit(actions, self.action12);
        actions = serialization::push_bit(actions, self.action13);
        actions = serialization::push_bit(actions, self.action14);
        actions = serialization::push_bit(actions, self.action15);
        actions = serialization::push_bit(actions, self.action16);
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
                action11: false,
                action12: false,
                action13: false,
                action14: false,
                action15: false,
                action16: false,
            },
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
                            match scancode {
                                17 => {
                                    // w
                                    self.input_state.action1 = pressed;
                                }
                                30 => {
                                    // a
                                }
                                31 => {
                                    // s
                                }
                                32 => {
                                    // d
                                }
                                57 => {
                                    // space
                                }
                                _ => {}
                            }
                        }
                        Events::Mouse(mouse_event) => {
                            //TODO: wire up sens + custom bindings
                            match mouse_event {
                                MouseEvents::CursorMove { xdelta, ydelta } => {
                                    const SENSITIVITY: f32 = 0.5;
                                    self.input_state.yaw_radians += xdelta * SENSITIVITY;
                                    self.input_state.pitch_radians += ydelta * SENSITIVITY;
                                }
                                _ => {
                                    //TODO:
                                }
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
            event_queue.add(Events::GfxView {
                pitch_radians: self.input_state.pitch_radians,
                yaw_radians: self.input_state.yaw_radians,
                roll_radians: self.input_state.roll_radians,
            })?;
        }

        Ok(())
    }
}

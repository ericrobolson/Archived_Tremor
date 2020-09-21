use crate::lib_core;
use lib_core::{serialization, time::Timer};

use crate::event_queue;
use event_queue::*;

/// Player input struct
#[derive(Copy, Clone, Debug)]
pub struct PlayerInput {
    pub player_input_id: usize,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl PlayerInput {
    pub fn new() -> Self {
        Self {
            player_input_id: 0,
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

#[derive(Copy, Clone, Debug)]
struct PlayerInputMapper {
    up_keycodes: [Option<u32>; 1],
    down_keycodes: [Option<u32>; 1],
    left_keycodes: [Option<u32>; 1],
    right_keycodes: [Option<u32>; 1],
}
impl PlayerInputMapper {
    pub fn new(player_id: usize) -> Self {
        if player_id == 0 {
            return Self {
                up_keycodes: [Some(17); 1],
                down_keycodes: [Some(31); 1],
                left_keycodes: [Some(30); 1],
                right_keycodes: [Some(32); 1],
            };
        }
        // TODO: need to link up input mapper for each player
        return Self {
            up_keycodes: [Some(72); 1],
            down_keycodes: [Some(80); 1],
            left_keycodes: [Some(75); 1],
            right_keycodes: [Some(77); 1],
        };
    }

    fn is_code(codes: &[Option<u32>; 1], keycode: u32) -> bool {
        for code in codes.iter().filter(|k| k.is_some()).map(|k| k.unwrap()) {
            if code == keycode {
                return true;
            }
        }
        return false;
    }

    fn is_up(&self, keycode: u32) -> bool {
        Self::is_code(&self.up_keycodes, keycode)
    }

    fn is_down(&self, keycode: u32) -> bool {
        Self::is_code(&self.down_keycodes, keycode)
    }

    fn is_left(&self, keycode: u32) -> bool {
        Self::is_code(&self.left_keycodes, keycode)
    }

    fn is_right(&self, keycode: u32) -> bool {
        Self::is_code(&self.right_keycodes, keycode)
    }
}

const MAX_LOCAL_PLAYERS: usize = 8;
pub struct ClientInputMapper {
    timer: Timer,
    input_states: [PlayerInput; MAX_LOCAL_PLAYERS],
    local_input_maps: [Option<PlayerInputMapper>; MAX_LOCAL_PLAYERS],
}

impl ClientInputMapper {
    pub fn new(poll_hz: u32) -> Self {
        let mut local_input_maps = [None; MAX_LOCAL_PLAYERS];
        let mut input_states = [PlayerInput::new(); MAX_LOCAL_PLAYERS];

        for (i, mut input_state) in input_states.iter_mut().enumerate() {
            input_state.player_input_id = i;
        }

        Self {
            timer: Timer::new(poll_hz),
            input_states: input_states,
            local_input_maps: local_input_maps,
        }
    }

    pub fn add_local_player(&mut self) -> Option<usize> {
        let mut player_id = None;
        for new_player_id in self
            .local_input_maps
            .iter()
            .enumerate()
            .filter(|(i, input_map)| input_map.is_none())
            .map(|(i, input_map)| i)
        {
            player_id = Some(new_player_id);
            break;
        }

        match player_id {
            Some(player_id) => {
                self.local_input_maps[player_id] = Some(PlayerInputMapper::new(player_id));
                return Some(player_id);
            }
            None => {}
        }

        None
    }

    pub fn execute(&mut self, event_queue: &mut EventQueue) -> Result<(), String> {
        //TODO: load keybindings from a config

        // Go thru events and update the state.
        for i in 0..event_queue.count() {
            match event_queue.events()[i] {
                Some((_duration, e)) => match e {
                    Events::Keyboard { scancode, pressed } => {
                        let pressed = match pressed {
                            ButtonState::Pressed => true,
                            ButtonState::Released => false,
                        };

                        for (player_id, input_mapper) in self
                            .local_input_maps
                            .iter()
                            .enumerate()
                            .filter(|(i, input_map)| input_map.is_some())
                            .map(|(i, input_map)| (i, input_map.unwrap()))
                        {
                            println!("keycode: {}", scancode);

                            if input_mapper.is_up(scancode) {
                                self.input_states[player_id].up = pressed;
                            }

                            if input_mapper.is_down(scancode) {
                                self.input_states[player_id].down = pressed;
                            }

                            if input_mapper.is_right(scancode) {
                                self.input_states[player_id].right = pressed;
                            }

                            if input_mapper.is_left(scancode) {
                                self.input_states[player_id].left = pressed;
                            }
                        }
                    }
                    _ => {}
                },
                None => {
                    break;
                }
            }
        }

        if self.timer.can_run() {
            // Add new input event to the queue with the current state
            for player_id in self
                .local_input_maps
                .iter()
                .enumerate()
                .filter(|(_i, input_map)| input_map.is_some())
                .map(|(i, _input_map)| i)
            {
                event_queue.add(Events::InputPoll(self.input_states[player_id]))?;
            }
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

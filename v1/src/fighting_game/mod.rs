use core::num;

use networking::rollback::prelude::*;

pub mod input_poller;

use game_math::f32::*;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum CharacterState {
    Idle,
    Walking,
    Dashing,
    Falling,
}
pub type RollbackGame = RollbackNetcode<GameState, Input>;

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Input {
    pub move_x_axis: i8,
    pub move_y_axis: i8,

    pub jump_pressed: bool,
    pub short_hop_pressed: bool,
    pub light_atk_pressed: bool,
    pub heavy_atk_pressed: bool,
    pub shield_pressed: bool,
    pub grab_pressed: bool,
}

impl GameInput for Input {}

#[derive(Copy, Clone, PartialEq)]
pub struct Aabb {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

const INPUT_BUFFER_SIZE: usize = 30;
const MIN_DASH_INPUT_BUFFER: usize = 3;
const MAX_DASH_INPUT_BUFFER: usize = 10;

#[derive(Clone, PartialEq)]
pub struct Character {
    input: Input,
    prev_inputs: [Input; INPUT_BUFFER_SIZE],
    pub prev_position: [f32; 3],
    pub position: [f32; 3],
    pub walk_speed: f32,
    pub dash_speed: f32,
    pub air_speed: f32,
    pub gravity_speed: f32,

    pub jumps: u8,
    pub max_jumps: u8,
    pub jump_frames: u8,
    pub max_jump_frames: u8,
    pub jump_speed: f32,

    pub state: CharacterState,

    pub push_boxes: Vec<Aabb>,
    pub hit_boxes: Vec<Aabb>,
    pub hurt_boxes: Vec<Aabb>,
    pub grab_boxes: Vec<Aabb>,
}

#[derive(Clone, PartialEq)]
pub struct GameState {
    pub characters: Vec<Character>,
    pub stage_aabbs: Vec<Aabb>,
    pub stage_position: [f32; 3],
}
impl RollbackGameState<Input> for GameState {
    fn new() -> Self {
        let mut state = Self {
            stage_position: [0.0, -1.0, 0.0],
            stage_aabbs: vec![Aabb {
                min: [-2.0, -0.25],
                max: [2.0, 0.25],
            }],
            characters: vec![
                Character {
                    state: CharacterState::Idle,
                    prev_inputs: [Input::default(); INPUT_BUFFER_SIZE],
                    input: Input::default(),
                    position: [0.5, 3.0, 0.0],
                    prev_position: [0.5, 3.0, 0.0],

                    walk_speed: 0.05,
                    dash_speed: 0.1,
                    air_speed: 0.2,
                    gravity_speed: 0.01,

                    jumps: 0,
                    max_jumps: 2,
                    jump_frames: 0,
                    max_jump_frames: 8,
                    jump_speed: 0.1,

                    push_boxes: vec![Aabb {
                        min: [-0.5, -0.5],
                        max: [0.5, 0.5],
                    }],
                    hit_boxes: vec![Aabb {
                        min: [-0.3, -0.35],
                        max: [0.3, 0.3],
                    }],
                    hurt_boxes: vec![],
                    grab_boxes: vec![],
                },
                /*
                Character {
                    state: CharacterState::Idle,
                    prev_inputs: [Input::default(); INPUT_BUFFER_SIZE],
                    input: Input::default(),
                    position: [-0.5, 0.0, 0.0],
                    walk_speed: 0.05,
                    dash_speed: 0.1,
                    push_boxes: vec![Aabb {
                        min: [-0.5, -0.5],
                        max: [0.5, 0.5],
                    }],
                    hit_boxes: vec![],
                    hurt_boxes: vec![],
                    grab_boxes: vec![],
                },*/
            ],
        };

        state
    }

    fn add_input(&mut self, player_id: PlayerId, input: Input) {
        let mut character = &mut self.characters[player_id as usize];
        character.input = input;

        // Handle input buffers
        {
            // Shift over previous inputs
            for i in 1..character.prev_inputs.len() {
                character.prev_inputs[i - 1] = character.prev_inputs[i];
            }

            // Copy over last input
            character.prev_inputs[character.prev_inputs.len() - 1] = character.input;
        }
    }

    fn tick(&mut self) {
        for character in self.characters.iter_mut() {
            character.prev_position = character.position;

            // Handle state transitions
            let input = map_input_to_logical_input(character.input);
            character.state = {
                if input.is_moving() == false {
                    CharacterState::Idle
                } else {
                    if is_jumpable(&character) {
                        // If last input was pressed and second to last input was not pressed, jump if possible
                        if character.input.jump_pressed
                            && !character.prev_inputs[INPUT_BUFFER_SIZE - 2].jump_pressed
                        {
                            character.jumps += 1;
                            character.jump_frames = character.max_jump_frames;
                        }
                    }

                    match character.state {
                        CharacterState::Idle => CharacterState::Walking,
                        CharacterState::Walking => {
                            let mut should_dash = false;
                            let mut empty_frames = 0;

                            for prev_input in character
                                .prev_inputs
                                .iter()
                                .rev()
                                .take(MAX_DASH_INPUT_BUFFER)
                            {
                                let logical_input = map_input_to_logical_input(*prev_input);
                                if !logical_input.left_held && !logical_input.right_held {
                                    empty_frames += 1;
                                } else {
                                    if empty_frames > MIN_DASH_INPUT_BUFFER {
                                        should_dash = true;
                                        break;
                                    }
                                }
                            }

                            match should_dash {
                                true => CharacterState::Dashing,
                                false => CharacterState::Walking,
                            }
                        }
                        CharacterState::Dashing => CharacterState::Dashing,
                        CharacterState::Falling => CharacterState::Falling,
                    }
                }
            };

            if character.state == CharacterState::Falling {
                character.position[1] -= character.gravity_speed;
            }

            // Do movement
            match character.state {
                CharacterState::Idle => {}
                CharacterState::Walking => {
                    move_character(character, character.walk_speed, input);
                }
                CharacterState::Dashing => {
                    move_character(character, character.dash_speed, input);
                }
                CharacterState::Falling => {
                    move_character(character, character.air_speed, input);
                }
            }

            // Jump movement
            if character.jump_frames > 0 {
                character.jump_frames -= 1;
                character.position[1] -= character.jump_speed;
            }
        }

        // Do collision checks and the ilk
        resolve_collisions(self);
    }
}

fn is_jumpable(character: &Character) -> bool {
    let can_jump = character.jumps < character.max_jumps;
    let valid_jump_state = character.state == CharacterState::Falling
        || character.state == CharacterState::Idle
        || character.state == CharacterState::Walking
        || character.state == CharacterState::Dashing;

    can_jump && valid_jump_state
}

fn move_character(character: &mut Character, speed: f32, input: LogicalInput) {
    if input.up_held {
        character.position[1] += speed;
    } else if input.down_held {
        character.position[1] -= speed;
    }

    if input.right_held {
        character.position[0] += speed;
    } else if input.left_held {
        character.position[0] -= speed;
    }
}

struct LogicalInput {
    up_held: bool,
    down_held: bool,
    left_held: bool,
    right_held: bool,
}

impl LogicalInput {
    fn is_moving(&self) -> bool {
        self.up_held || self.down_held || self.left_held || self.right_held
    }
}

fn map_input_to_logical_input(input: Input) -> LogicalInput {
    let up_held = input.move_y_axis > 0;
    let down_held = input.move_y_axis < 0;

    let right_held = input.move_x_axis > 0;
    let left_held = input.move_x_axis < 0;

    LogicalInput {
        up_held,
        down_held,
        right_held,
        left_held,
    }
}

fn resolve_collisions(state: &mut GameState) {
    for character in state.characters.iter_mut() {
        for push_box in &character.push_boxes {}
    }
}

type GameInput = super::Input;

use portia::input::Input;

pub struct InputPoller {
    input: GameInput,
    up_held: bool,
    down_held: bool,
    right_held: bool,
    left_held: bool,
    jump_held: bool,
    frame: usize,
}

impl InputPoller {
    pub fn new() -> Self {
        Self {
            up_held: false,
            down_held: false,
            right_held: false,
            left_held: false,
            jump_held: false,
            input: GameInput::default(),
            frame: 0,
        }
    }

    pub fn poll(&mut self, inputs: &Vec<portia::input::Input>) -> GameInput {
        self.frame = self.frame.wrapping_add(1);

        // Simple polling for input.
        // TODO: make configurable.
        for input in inputs {
            match input {
                Input::Key { key, state } => match key {
                    portia::input::Key::Up => match state {
                        portia::input::PressState::Released => {
                            self.up_held = false;
                        }
                        portia::input::PressState::Pressed => {
                            self.up_held = true;
                        }
                    },
                    portia::input::Key::Down => match state {
                        portia::input::PressState::Released => {
                            self.down_held = false;
                        }
                        portia::input::PressState::Pressed => {
                            self.down_held = true;
                        }
                    },
                    portia::input::Key::Left => match state {
                        portia::input::PressState::Released => {
                            self.left_held = false;
                        }
                        portia::input::PressState::Pressed => {
                            self.left_held = true;
                        }
                    },
                    portia::input::Key::Right => match state {
                        portia::input::PressState::Released => {
                            self.right_held = false;
                        }
                        portia::input::PressState::Pressed => {
                            self.right_held = true;
                        }
                    },
                    portia::input::Key::Space => match state {
                        portia::input::PressState::Released => {
                            self.jump_held = false;
                        }
                        portia::input::PressState::Pressed => {
                            self.jump_held = true;
                        }
                    },
                },
                _ => {}
            }
        }

        let axis_max = i8::MAX;
        let axis_min = i8::MIN;

        if self.up_held {
            self.input.move_y_axis = axis_max;
        } else if self.down_held {
            self.input.move_y_axis = axis_min;
        } else {
            self.input.move_y_axis = 0;
        }

        if self.right_held {
            self.input.move_x_axis = axis_max;
        } else if self.left_held {
            self.input.move_x_axis = axis_min;
        } else {
            self.input.move_x_axis = 0;
        }

        if self.jump_held {
            self.input.jump_pressed = true;
        }

        self.input
    }
}

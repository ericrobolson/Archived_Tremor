use crate::lib_core;
use lib_core::time::{Clock, Duration};

#[derive(Copy, Clone, Debug)]
pub enum ButtonState {
    Pressed,
    Released,
}

#[derive(Copy, Clone, Debug)]
pub enum MouseEvents {
    LeftButton(ButtonState),
    RightButton(ButtonState),
    MiddleButton(ButtonState),
    OtherButton(u8, ButtonState),
    CursorMove { xdelta: f32, ydelta: f32 },
    MouseWheel { ydelta: f32 },
}

#[derive(Copy, Clone, Debug)]
pub enum Events {
    Keyboard { pressed: ButtonState, scancode: u32 },
    Mouse(MouseEvents),
    Socket,
}

const EVENT_SIZE: usize = 256;

pub struct EventQueue {
    clock: Clock,
    events: [Option<(Duration, Events)>; EVENT_SIZE],
    index: usize,
}

impl EventQueue {
    pub const EVENT_SIZE: usize = EVENT_SIZE;

    pub fn new() -> Self {
        Self {
            clock: Clock::new(),
            events: [None; Self::EVENT_SIZE],
            index: 0,
        }
    }

    pub fn add(&mut self, event: Events) -> Result<(), String> {
        if self.index >= Self::EVENT_SIZE {
            return Err("Input buffer overflow".into());
        }
        self.events[self.index] = Some((self.clock.elapsed(), event));
        self.index += 1;

        Ok(())
    }

    pub fn events(&self) -> &[Option<(Duration, Events)>] {
        &self.events
    }

    pub fn clear(&mut self) {
        for i in 0..Self::EVENT_SIZE {
            self.events[i] = None;
        }

        self.index = 0;
    }
}

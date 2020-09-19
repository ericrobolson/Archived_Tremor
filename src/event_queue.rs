use crate::lib_core;
use crate::network::connection_layer::ConnectionId;
use crate::network::{Packet, Sequence, SocketAddr};
use lib_core::{input::PlayerInput, time::Clock, time::Duration};

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
pub enum SocketEvents {
    Recieved(Packet, SocketAddr),
    ToSend(Packet, SocketAddr),
    Ack(Sequence, SocketAddr),
    Dropped(Sequence, SocketAddr),
}

#[derive(Copy, Clone, Debug)]
pub enum Events {
    Keyboard {
        pressed: ButtonState,
        scancode: u32,
    },
    Mouse(MouseEvents),
    InputPoll(PlayerInput),
    GfxView {
        pitch_radians: f32,
        yaw_radians: f32,
        roll_radians: f32,
    },
    Socket(SocketEvents),
}

const EVENT_SIZE: usize = 256;

pub struct EventQueue {
    clock: Clock,
    events: Vec<Option<(Duration, Events)>>,
    index: usize,
    count: usize,
}

impl EventQueue {
    pub const EVENT_SIZE: usize = EVENT_SIZE;

    pub fn new() -> Self {
        let mut events = Vec::with_capacity(Self::EVENT_SIZE);
        for _ in 0..EVENT_SIZE {
            events.push(None);
        }

        Self {
            clock: Clock::new(),
            events: events,
            index: 0,
            count: 0,
        }
    }

    pub fn add(&mut self, event: Events) -> Result<(), String> {
        if self.index >= Self::EVENT_SIZE {
            return Err("Input buffer overflow".into());
        }
        self.events[self.index] = Some((self.clock.elapsed(), event));
        self.index += 1;
        self.count += 1;

        Ok(())
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn events(&self) -> &[Option<(Duration, Events)>] {
        &self.events
    }

    pub fn clear(&mut self) {
        for i in 0..Self::EVENT_SIZE {
            self.events[i] = None;
        }

        self.index = 0;
        self.count = 0;
    }
}

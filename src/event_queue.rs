#[derive(Copy, Clone)]
pub enum Events {
    Keyboard,
    Mouse,
    Socket,
}

const EVENT_SIZE: usize = 256;

pub struct EventQueue {
    events: [Option<Events>; EVENT_SIZE],
    index: usize,
}

impl EventQueue {
    pub const EVENT_SIZE: usize = EVENT_SIZE;

    pub fn new() -> Self {
        Self {
            events: [None; Self::EVENT_SIZE],
            index: 0,
        }
    }

    pub fn add(&mut self, event: Events) -> Result<(), String> {
        if self.index >= Self::EVENT_SIZE {
            return Err("Input buffer overflow".into());
        }
        self.events[self.index] = Some(event);
        self.index += 1;

        Ok(())
    }

    pub fn events(&self) -> &[Option<Events>] {
        &self.events
    }

    pub fn clear(&mut self) {
        for mut event in self.events.iter() {
            event = &None;
        }

        self.index = 0;
    }
}

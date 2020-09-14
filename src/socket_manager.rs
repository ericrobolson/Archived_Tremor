use crate::event_queue;
use event_queue::*;

pub struct SocketManager {}

impl SocketManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read(&mut self, event_queue: &mut EventQueue) -> Result<(), String> {
        println!("Got socket events");

        Ok(())
    }
}

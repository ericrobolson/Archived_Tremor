use crate::event_queue;
use event_queue::*;

pub struct SocketManager {}

impl SocketManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn poll(
        &mut self,
        event_queue: &mut EventQueue,
        socket_out_queue: &EventQueue,
    ) -> Result<(), String> {
        // Queue up input to send to server
        // Send things to clients

        Ok(())
    }
}

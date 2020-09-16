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

        for i in 0..socket_out_queue.count() {
            match socket_out_queue.events()[i] {
                Some((_, e)) => match e {
                    Events::InputPoll(poll) => {
                        println!("Poll: {:?}", poll);
                    }
                    _ => {}
                },
                None => {
                    break;
                }
            }
        }

        Ok(())
    }
}

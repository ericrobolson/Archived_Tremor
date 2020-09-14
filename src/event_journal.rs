use crate::event_queue;
use event_queue::*;

pub struct EventJournal {}

impl EventJournal {
    pub fn new() -> Self {
        Self {}
    }

    pub fn dump(&mut self, event_queue: &EventQueue) -> Result<(), String> {
        println!("Dumped event queue");

        Ok(())
    }
}

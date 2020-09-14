use crate::event_queue;
use event_queue::*;

pub struct EventJournal {}

impl EventJournal {
    pub fn new() -> Self {
        Self {}
    }

    pub fn dump(&mut self, event_queue: &EventQueue) -> Result<(), String> {
        for item in event_queue.events() {
            match item {
                Some((timestamp, event)) => {
                    println!("{:?}: {:?}", timestamp, event);
                }
                None => {}
            }
        }

        Ok(())
    }
}

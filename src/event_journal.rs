use crate::event_queue;
use event_queue::*;

use crate::lib_core;
use lib_core::time;

pub struct EventJournal {}

impl EventJournal {
    pub fn new() -> Self {
        // Create file
        println!("{}", time::sys_time());
        Self {}
    }

    pub fn dump(&mut self, event_queue: &EventQueue) -> Result<(), String> {
        for item in event_queue.events() {
            match item {
                Some((timestamp, event)) => match event {
                    Events::GfxView {
                        pitch_radians,
                        yaw_radians,
                        roll_radians,
                    } => {}
                    Events::Mouse(_) => {}
                    Events::InputPoll(_) => {}
                    _ => {
                        println!("{:?}: {:?}", timestamp, event);
                    }
                },
                None => {}
            }
        }

        Ok(())
    }
}

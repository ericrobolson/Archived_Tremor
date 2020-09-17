use std::net::UdpSocket;
use std::time::Duration;

use crate::event_queue;
use crate::lib_core::{time::Timer, LookUpGod};
use crate::network::Packet;

use event_queue::*;

pub struct StreamManager {}

impl StreamManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn queue_outbound(&mut self, event_queue: &EventQueue, socket_out_queue: &mut EventQueue) {}

    pub fn parse_inbound(&mut self, event_queue: &mut EventQueue) {}
}

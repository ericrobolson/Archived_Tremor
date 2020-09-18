use crate::event_queue;
use crate::lib_core::{time::Timer, LookUpGod};
use crate::network::{Packet, Sequence};

use event_queue::*;

pub struct StreamManager {
    send_timer: Timer,
    next_packet_id: Sequence,
}

impl StreamManager {
    pub fn new() -> Self {
        Self {
            send_timer: Timer::new(20),
            next_packet_id: 1,
        }
    }

    pub fn queue_outbound(
        &mut self,
        event_queue: &EventQueue,
        socket_out_queue: &mut EventQueue,
    ) -> Result<(), String> {
        if self.send_timer.can_run() {
            let mut packet = Packet::new();
            packet.write_f32(345.321);
            packet.set_sequence(self.next_packet_id);

            self.next_packet_id = self.next_packet_id.wrapping_add(1);

            socket_out_queue.add(Events::Socket(SocketEvents::ToSend(packet)))?;
        }

        Ok(())
    }

    pub fn parse_inbound(&mut self, event_queue: &mut EventQueue) {}
}

use std::net::UdpSocket;
use std::time::Duration;

use crate::event_queue;
use crate::lib_core::{data_structures::CircularBuffer, math, time::Timer, LookUpGod};
use crate::network;
use network::{Packet, Sequence};

use event_queue::*;

pub type SocketAddr = std::net::SocketAddr;
pub type ConnectionId = u8;

const CIRCLE_BUFFER_LEN: usize = network::ACK_BIT_LENGTH as usize;

pub struct ConnectionManager {
    max_remote_connections: usize,
    send_timer: Timer,
}

impl ConnectionManager {
    pub fn new(max_remote_connections: usize) -> Self {
        //TODO: a set number of clients / spectators. Basically a single class to manage all connections
        //TODO: Initialize a ConnectionLayer per client so that it can determine what to send and what to do with dropped packets
        Self {
            max_remote_connections: max_remote_connections,
            send_timer: Timer::new(30),
        }
    }

    pub fn write_all(
        &mut self,
        event_queue: &EventQueue,
        socket_out_queue: &mut EventQueue,
    ) -> Result<(), String> {
        if self.send_timer.can_run() {}
        Ok(())
    }

    pub fn read_all(&mut self, event_queue: &mut EventQueue) -> Result<(), String> {
        // Read in all packets, delegating them to the proper ConnectionLayer
        Ok(())
    }
}

struct ConnectionLayer {
    connection_id: ConnectionId,
    remote_addr: SocketAddr,
    read_timer: Timer,
    write_timer: Timer,
    sent_packet_buffer: CircularBuffer<Option<bool>>,
    recieved_packet_buffer: CircularBuffer<Option<bool>>,
    last_recieved_packet: network::Sequence,
    next_packet_id: Sequence,
}

impl ConnectionLayer {
    pub fn new(remote_addr: SocketAddr, connection_id: ConnectionId) -> Self {
        Self {
            connection_id: connection_id,
            read_timer: Timer::new(30),
            write_timer: Timer::new(30),
            remote_addr: remote_addr,
            sent_packet_buffer: CircularBuffer::new(CIRCLE_BUFFER_LEN, None),
            recieved_packet_buffer: CircularBuffer::new(CIRCLE_BUFFER_LEN, None),
            last_recieved_packet: 0,
            next_packet_id: 0,
        }
    }

    pub fn new_packet(&mut self) -> Packet {
        let mut packet = Packet::new();
        packet.set_sequence(self.next_packet_id);
        self.next_packet_id = self.next_packet_id.wrapping_add(1);

        packet
    }

    pub fn send(
        &mut self,
        packet: Packet,
        event_queue: &EventQueue,
        socket_out_queue: &mut EventQueue,
    ) -> Result<(), String> {
        let mut packet = packet;

        // Insert entry for current packet into the sent packet sequence buffer saying it hasn't been ackd
        self.sent_packet_buffer
            .insert(packet.sequence() as usize, Some(false));

        // Generate ack and ack_bits from contents of recieved packet buffer and most recent packet sequence number
        {
            packet.set_ack(self.last_recieved_packet);

            // Calculate ack_bits for last 32 packets
            let mut ack_bits = 0;
            let last_recieved_packet: usize = self.last_recieved_packet as usize;

            for i in 0..network::ACK_BIT_LENGTH {
                if self
                    .recieved_packet_buffer
                    .item(math::wrap_op_usize(
                        last_recieved_packet,
                        i,
                        math::Ops::Subtract,
                    ))
                    .is_some()
                {
                    // Toggle the bit at i to true
                    let value = 1 << i;
                    ack_bits = ack_bits | value;
                } else {
                    // No packet to ack, so continue to next entry
                    continue;
                }
            }

            packet.set_ack_bits(ack_bits);
        }

        socket_out_queue.add(Events::Socket(SocketEvents::ToSend(
            packet,
            self.remote_addr,
        )))?;
        Ok(())
    }

    pub fn recieve(&mut self, event_queue: &mut EventQueue) -> Result<(), String> {
        let mut results = Vec::with_capacity(64); // TODO: come up with actual size?

        for (_duration, event) in event_queue
            .events()
            .iter()
            .filter(|e| e.is_some())
            .map(|e| e.unwrap())
        {
            match event {
                Events::Socket(socket_event) => match socket_event {
                    SocketEvents::Recieved(packet, addr) => {
                        if addr == self.remote_addr {
                            let sequence = packet.sequence() as usize;
                            // Read in sequence from the packet header
                            // If sequence is more recent than the previous most recent received packet sequence number, update the most recent received packet sequence number
                            if is_packet_newer(packet.sequence(), self.last_recieved_packet) {
                                // Unfortunately sometimes packets arrive out of order and some are lost.
                                // To fix issues where old buffer entries stick around too long, walk the entries between the previous highest and the current one and clear them
                                if self.last_recieved_packet <= packet.sequence() {
                                    for i in self.last_recieved_packet as usize..sequence {
                                        self.recieved_packet_buffer.insert(i, None);

                                        results.push(Events::Socket(SocketEvents::Dropped(
                                            i as Sequence,
                                            self.connection_id,
                                        )));
                                    }
                                }

                                self.last_recieved_packet = packet.sequence();
                            }
                            // Insert an entry for this packet in the received packet sequence buffer
                            self.recieved_packet_buffer.insert(sequence, Some(true));

                            // Decode the set of acked packet sequence numbers from ack and ack_bits in the packet header.
                            // Iterate across all acked packet sequence numbers and for any packet that is not already acked add ack event and mark that packet as acked in the sent packet sequence buffer.
                            for i in 0..network::ACK_BIT_LENGTH {
                                let index = math::wrap_op_usize(sequence, i, math::Ops::Subtract);

                                let value = 1 << i; // Select the current ack_bit
                                let is_ackd = (packet.ack_bits() & value) > 0;

                                if is_ackd {
                                    let sent_value = *self.sent_packet_buffer.item(index);
                                    match sent_value {
                                        Some(false) => {
                                            // Set it to true and add it to the event queue
                                            results.push(Events::Socket(SocketEvents::Ack(
                                                index as network::Sequence,
                                                addr,
                                            )));

                                            self.sent_packet_buffer.insert(index, Some(true));
                                        }
                                        Some(true) => {}
                                        None => {}
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        for result in results {
            event_queue.add(result)?;
        }

        Ok(())
    }
}

fn is_packet_newer(
    packet_sequence: network::Sequence,
    old_packet_sequence: network::Sequence,
) -> bool {
    if packet_sequence > old_packet_sequence {
        return true;
    }

    // Check for wrapping
    //TODO: if having bugs with sending around the boundaries, revisit. For now it's simply dumb and arbitrary.
    const CIRCLE_BUFFER_LEN_CONVERTED: network::Sequence = CIRCLE_BUFFER_LEN as network::Sequence;
    if old_packet_sequence > network::MAX_SEQUENCE_VALUE - CIRCLE_BUFFER_LEN_CONVERTED
        && packet_sequence < CIRCLE_BUFFER_LEN_CONVERTED
    {
        return true;
    }

    false
}

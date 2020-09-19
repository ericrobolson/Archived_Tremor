use std::net::UdpSocket;
use std::time::Duration;

use crate::event_queue;
use crate::lib_core::{data_structures::CircularBuffer, math, time::Timer, LookUpGod};
use crate::network;
use network::Packet;

use event_queue::*;

const CIRCLE_BUFFER_LEN: usize = network::ACK_BIT_LENGTH as usize;
pub type SocketAddr = std::net::SocketAddr;

pub struct SocketManager {
    socket: UdpSocket,
    timer: Timer,
}

impl SocketManager {
    pub fn new(server_addr: &'static str) -> Result<Self, String> {
        let socket = match UdpSocket::bind(server_addr) {
            Ok(s) => s,
            Err(e) => {
                return Err(format!("{:?}", e));
            }
        };

        match socket.set_read_timeout(Some(Duration::new(0, 1))) {
            Ok(s) => s,
            Err(e) => {
                return Err(format!("{:?}", e));
            }
        }

        Ok(Self {
            socket: socket,
            timer: Timer::new(60),
        })
    }

    pub fn poll(
        &mut self,
        lug: &LookUpGod,
        event_queue: &mut EventQueue,
        socket_out_queue: &EventQueue,
    ) -> Result<(), String> {
        if self.timer.can_run() {
            // Send outbound
            for (_duration, event) in socket_out_queue
                .events()
                .iter()
                .filter(|e| e.is_some())
                .map(|e| e.unwrap())
            {
                match event {
                    Events::Socket(socket_msg) => match socket_msg {
                        SocketEvents::ToSend(packet, addr) => {
                            send_socket(lug, &mut self.socket, &packet, addr)?;
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }

            // Read inbound
            let mut still_reading = true;
            while still_reading {
                match try_read_socket(lug, &mut self.socket)? {
                    Some((packet, addr)) => {
                        // Add the packet to the event queue
                        event_queue.add(Events::Socket(SocketEvents::Recieved(packet, addr)))?;
                    }
                    None => {
                        still_reading = false;
                    }
                }
            }
        }

        Ok(())
    }
    /*
    pub fn poll_old(
        &mut self,
        lug: &LookUpGod,
        event_queue: &mut EventQueue,
        socket_out_queue: &EventQueue,
    ) -> Result<(), String> {
        // Outbound messages
        for i in 0..socket_out_queue.count() {
            match socket_out_queue.events()[i] {
                Some((_, e)) => match e {
                    Events::Socket(socket_msg) => match socket_msg {
                        SocketEvents::ToSend(packet, addr) => {
                            let mut packet = packet;

                            // Insert entry for current packet into the sent packet sequence buffer saying it hasn't been ackd
                            self.sent_packet_buffer
                                .insert(packet.sequence() as usize, Some(false));

                            // Generate ack and ack_bits from contents of recieved packet buffer and most recent packet sequence number
                            {
                                packet.set_ack(self.last_recieved_packet);

                                // Calculate ack_bits for last 32 packets
                                let mut ack_bits = 0;
                                let last_recieved_packet: usize =
                                    self.last_recieved_packet as usize;

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

                            send_socket(lug, &mut self.socket, &packet, addr)?;
                        }
                        _ => {}
                    },
                    _ => {}
                },
                None => {
                    break;
                }
            }
        }

        // Inbound messages
        if self.read_timer.can_run() {
            // TODO: maybe loop here so long as it can read a packet? Not sure we want to read both client + server stuff here
            match try_read_socket(lug, &mut self.socket)? {
                Some((packet, addr)) => {
                    let sequence = packet.sequence() as usize;
                    // Read in sequence from the packet header
                    // If sequence is more recent than the previous most recent received packet sequence number, update the most recent received packet sequence number
                    if is_packet_newer(packet.sequence(), self.last_recieved_packet) {
                        // Unfortunately sometimes packets arrive out of order and some are lost.
                        // To fix issues where old buffer entries stick around too long, walk the entries between the previous highest and the current one and clear them

                        // Check if at the boundary and need to wipe old messages
                        if self.last_recieved_packet <= packet.sequence() {
                            for i in self.last_recieved_packet as usize..sequence {
                                self.recieved_packet_buffer.insert(i, None);
                            }
                        } else {
                            // Wipe out old values
                            for i in self.last_recieved_packet as usize
                                ..network::MAX_SEQUENCE_VALUE as usize
                            {
                                self.recieved_packet_buffer.insert(i, None);
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
                                    event_queue.add(Events::Socket(SocketEvents::Ack(
                                        index as network::Sequence,
                                        addr,
                                    )))?;

                                    self.sent_packet_buffer.insert(index, Some(true));
                                }
                                Some(true) => {}
                                None => {}
                            }
                        }
                    }

                    // Add the packet to the event queue
                    event_queue.add(Events::Socket(SocketEvents::Recieved(packet, addr)))?;
                }
                None => {}
            }
        }

        Ok(())
    }
    */
}

fn send_socket(
    lug: &LookUpGod,
    socket: &mut UdpSocket,
    packet: &Packet,
    addr: std::net::SocketAddr,
) -> Result<(), String> {
    match socket.send_to(&packet.to_network_bytes(lug), addr) {
        Ok(_) => {}
        Err(e) => {
            return Err(format!("{:?}", e));
        }
    }

    Ok(())
}

fn try_read_socket(
    lug: &LookUpGod,
    socket: &mut UdpSocket,
) -> Result<Option<(Packet, SocketAddr)>, String> {
    let mut buf = [0; Packet::TOTAL_PACKET_LEN];
    match socket.recv_from(&mut buf) {
        Ok((_size, addr)) => {
            let packet = Packet::from_bytes(lug, buf);
            match packet {
                Some(packet) => {
                    return Ok(Some((packet, addr)));
                }
                None => {
                    return Ok(None);
                }
            }
        }
        Err(e) => match e.kind() {
            std::io::ErrorKind::WouldBlock => {}
            std::io::ErrorKind::TimedOut => {}
            _ => {
                return Err(format!("{}", e));
            }
        },
    }

    Ok(None)
}

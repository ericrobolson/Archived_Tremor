use std::net::UdpSocket;
use std::time::Duration;

use crate::event_queue;
use crate::lib_core::{data_structures::CircularBuffer, math, time::Timer, LookUpGod};
use crate::network;
use network::Packet;

use event_queue::*;

const CIRCLE_BUFFER_LEN: usize = network::ACK_BIT_LENGTH as usize;

pub struct SocketManager {
    read_timer: Timer,
    write_timer: Timer,
    client_socket: UdpSocket,
    server_socket: UdpSocket,
    sent_packet_buffer: CircularBuffer<Option<bool>>,
    recieved_packet_buffer: CircularBuffer<Option<bool>>,
    last_recieved_packet: network::Sequence,
}

impl SocketManager {
    pub fn new(server_addr: &'static str, client_addr: &'static str) -> Result<Self, String> {
        let client = match UdpSocket::bind(client_addr) {
            Ok(s) => s,
            Err(e) => {
                return Err(format!("{:?}", e));
            }
        };

        let server = match UdpSocket::bind(server_addr) {
            Ok(s) => s,
            Err(e) => {
                return Err(format!("{:?}", e));
            }
        };

        match client.set_read_timeout(Some(Duration::new(0, 1))) {
            Ok(s) => s,
            Err(e) => {
                return Err(format!("{:?}", e));
            }
        }

        match server.set_read_timeout(Some(Duration::new(0, 1))) {
            Ok(s) => s,
            Err(e) => {
                return Err(format!("{:?}", e));
            }
        }

        Ok(Self {
            read_timer: Timer::new(30),
            write_timer: Timer::new(30),
            client_socket: client,
            server_socket: server,
            sent_packet_buffer: CircularBuffer::new(CIRCLE_BUFFER_LEN, None),
            recieved_packet_buffer: CircularBuffer::new(CIRCLE_BUFFER_LEN, None),
            last_recieved_packet: 0,
        })
    }

    pub fn poll(
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
                        SocketEvents::ToSend(packet) => {
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

                            let serverAddr = self.server_socket.local_addr().unwrap();
                            send_socket(lug, &mut self.client_socket, &packet, serverAddr)?;
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
            let mut incoming_socket = &mut self.server_socket;
            match try_read_socket(lug, &mut incoming_socket)? {
                Some(packet) => {
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
                                    println!("ACKD: {}", index);
                                    // Set it to true and add it to the event queue
                                    event_queue.add(Events::Socket(SocketEvents::Ack(
                                        index as network::Sequence,
                                    )))?;

                                    self.sent_packet_buffer.insert(index, Some(true));
                                }
                                Some(true) => {}
                                None => {}
                            }
                        }
                    }

                    // Add the packet to the event queue
                    println!("Recieved: {:?}", packet);
                    event_queue.add(Events::Socket(SocketEvents::Recieved(packet)))?;
                }
                None => {}
            }
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

fn try_read_socket(lug: &LookUpGod, socket: &mut UdpSocket) -> Result<Option<Packet>, String> {
    let mut buf = [0; Packet::TOTAL_PACKET_LEN];
    match socket.recv(&mut buf) {
        Ok(_) => {
            return Ok(Packet::from_bytes(lug, buf));
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

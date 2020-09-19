use crate::event_queue;
use crate::lib_core::{data_structures::CircularBuffer, math};
use crate::network;
use network::{Packet, Sequence};

use event_queue::*;

pub type SocketAddr = std::net::SocketAddr;
pub type ConnectionId = u8;

const CIRCLE_BUFFER_LEN: usize = network::ACK_BIT_LENGTH as usize;

pub struct ConnectionManager {
    max_remote_connections: usize,
    connections: Vec<VirtualConnection>,
}

impl ConnectionManager {
    pub fn new(max_remote_connections: usize) -> Self {
        Self {
            max_remote_connections: max_remote_connections,
            connections: Vec::with_capacity(max_remote_connections),
        }
    }

    pub fn write_all(
        &mut self,
        event_queue: &EventQueue,
        socket_out_queue: &mut EventQueue,
    ) -> Result<(), String> {
        // These are not from the out_queue, but passed in to the event queue from the stream manager
        //TODO: Instead, should the stream manager live here and do it's calculations?
        let packets_to_send = event_queue.events().iter().filter_map(|e| match e {
            Some((_duration, event)) => match event {
                Events::Socket(SocketEvents::ToSend(packet, addr)) => Some((*packet, *addr)),
                _ => None,
            },
            None => None,
        });
        for (packet, addr) in packets_to_send {
            for connection in self.connections.iter_mut() {
                if addr != connection.remote_addr {
                    continue;
                }

                connection.send(packet, socket_out_queue)?;
            }
        }
        Ok(())
    }

    fn try_add_connection(&mut self, socket_addr: SocketAddr) -> Result<usize, String> {
        if self.connections.len() >= self.max_remote_connections {
            return Err(
                "Max clients exceeded! TODO: Resolve + send 'unable to join packet'".into(),
            );
        }
        let connection_index = self.connections.len();
        let connection = VirtualConnection::new(socket_addr);

        self.connections.push(connection);

        Ok(connection_index)
    }

    pub fn read_all(&mut self, event_queue: &mut EventQueue) -> Result<(), String> {
        // Read in all packets, delegating them to the proper ConnectionLayer. Add a new connection layer if no existing ones match the address
        let recieved_packets: Vec<(Packet, SocketAddr)> = event_queue
            .events()
            .iter()
            .filter_map(|e| match e {
                Some((_duration, event)) => match event {
                    Events::Socket(SocketEvents::Recieved(packet, addr)) => Some((*packet, *addr)),
                    _ => None,
                },
                None => None,
            })
            .collect::<Vec<(Packet, SocketAddr)>>();

        for (packet, addr) in recieved_packets {
            // Initialize a ConnectionLayer per client so that it can determine what to send and what to do with dropped packets
            let mut found_connection = false;
            for connection in self.connections.iter_mut() {
                // If address does exist, pass packet to client
                if connection.remote_addr == addr {
                    connection.recieve(packet, event_queue)?;
                    found_connection = true;
                    break;
                }
            }
            // If address doesn't exist, try to add a new client and pass packet
            if !found_connection {
                let connection_index = self.try_add_connection(addr)?;
                self.connections[connection_index].recieve(packet, event_queue)?;
            }
        }
        Ok(())
    }
}

struct VirtualConnection {
    remote_addr: SocketAddr,
    sent_packet_buffer: CircularBuffer<Option<bool>>,
    recieved_packet_buffer: CircularBuffer<Option<bool>>,
    last_recieved_packet: network::Sequence,
    next_packet_id: Sequence,
}

impl VirtualConnection {
    pub fn new(remote_addr: SocketAddr) -> Self {
        VirtualConnection {
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

    pub fn recieve(&mut self, packet: Packet, event_queue: &mut EventQueue) -> Result<(), String> {
        let mut results = Vec::with_capacity(64); // TODO: come up with actual size?

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
                        self.remote_addr,
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
                            self.remote_addr,
                        )));

                        self.sent_packet_buffer.insert(index, Some(true));
                    }
                    Some(true) => {}
                    None => {}
                }
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

use std::net::UdpSocket;
use std::time::Duration;

use crate::encryption::{Crc32, CRC32};
use crate::network;
use network::Packet;

pub type SocketAddr = std::net::SocketAddr;

pub struct SocketManager {
    socket: UdpSocket,
}

impl SocketManager {
    pub fn new(server_addr: &'static str) -> Result<Self, String> {
        let socket = match UdpSocket::bind(server_addr) {
            Ok(s) => s,
            Err(e) => {
                return Err(format!("{:?}", e));
            }
        };

        println!("Addr: {:?}", socket.local_addr().unwrap().ip());

        match socket.set_read_timeout(Some(Duration::new(0, 1))) {
            Ok(s) => s,
            Err(e) => {
                return Err(format!("{:?}", e));
            }
        }

        Ok(Self { socket: socket })
    }

    pub fn poll(
        &mut self,
        socket_out_queue: &Vec<(Packet, SocketAddr)>,
    ) -> Result<Vec<(Packet, SocketAddr)>, String> {
        let crc32 = &CRC32;

        // Send outbound
        for (packet, addr) in socket_out_queue {
            send_socket(&crc32, &mut self.socket, &packet, *addr)?;
        }

        // Read inbound
        let mut still_reading = true;
        let mut received_packets = vec![];
        while still_reading {
            match try_read_socket(&crc32, &mut self.socket)? {
                Some((packet, addr)) => {
                    received_packets.push((packet, addr));
                }
                None => {
                    still_reading = false;
                }
            }
        }

        Ok(received_packets)
    }
}

fn send_socket(
    crc32: &Crc32,
    socket: &mut UdpSocket,
    packet: &Packet,
    addr: std::net::SocketAddr,
) -> Result<(), String> {
    match socket.send_to(&packet.to_network_bytes(crc32), addr) {
        Ok(_) => {}
        Err(e) => {
            return Err(format!("{:?}", e));
        }
    }

    Ok(())
}

fn try_read_socket(
    crc32: &Crc32,
    socket: &mut UdpSocket,
) -> Result<Option<(Packet, SocketAddr)>, String> {
    let mut buf = [0; Packet::TOTAL_PACKET_LEN];
    match socket.recv_from(&mut buf) {
        Ok((_size, addr)) => {
            let packet = Packet::from_bytes(crc32, buf);
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

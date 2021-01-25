use std::net::UdpSocket;
use std::time::Duration;

use crate::event_queue;
use crate::lib_core::{time::Timer, LookUpGod};
use crate::network;
use network::Packet;

use event_queue::*;

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

        println!("Addr: {:?}", socket.local_addr().unwrap().ip());

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

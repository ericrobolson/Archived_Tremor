use std::net::UdpSocket;
use std::time::Duration;

use crate::event_queue;
use crate::lib_core::{time::Timer, LookUpGod};
use crate::network::Packet;

use event_queue::*;

pub struct SocketManager {
    read_timer: Timer,
    write_timer: Timer,
    client_socket: UdpSocket,
    server_socket: UdpSocket,
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
        })
    }

    pub fn poll(
        &mut self,
        lug: &LookUpGod,
        event_queue: &mut EventQueue,
        socket_out_queue: &EventQueue,
    ) -> Result<(), String> {
        // Outbound messages
        //TODO: send out on a regular schedule instead of as fast as possible; maybe queue up messages?
        for i in 0..socket_out_queue.count() {
            match socket_out_queue.events()[i] {
                Some((_, e)) => match e {
                    Events::InputPoll(poll) => {
                        let serverAddr = self.server_socket.local_addr().unwrap();
                        send_socket(lug, &mut self.client_socket, &Packet::new(), serverAddr)?;
                    }
                    _ => {}
                },
                None => {
                    break;
                }
            }
        }

        // Inbound messages
        if self.read_timer.can_run() {
            // TODO: maybe loop here?
            let server_msg = try_read_socket(lug, &mut self.client_socket)?;
            let client_msgs = try_read_socket(lug, &mut self.server_socket)?;
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

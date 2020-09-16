use crate::event_queue;
use crate::lib_core::time::Timer;
use event_queue::*;
use std::time::Duration;

use std::net::UdpSocket;

type PacketByteLength = u32;
const PACKET_BYTE_LENGTH_BYTE_SIZE: usize = 420; // BLAZE IT (in actually, going off of http://ithare.com/64-network-dos-and-donts-for-game-engines-part-v-udp/ to limit the size to under 500 for MTU purposes)
struct Packet {
    data: [u8; PACKET_BYTE_LENGTH_BYTE_SIZE],
}

pub struct SocketManager {
    read_timer: Timer,
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
            client_socket: client,
            server_socket: server,
        })
    }

    pub fn poll(
        &mut self,
        event_queue: &mut EventQueue,
        socket_out_queue: &EventQueue,
    ) -> Result<(), String> {
        // Outbound messages
        //TODO: send out on a regular schedule instead of as fast as possible; maybe queue up messages?
        for i in 0..socket_out_queue.count() {
            match socket_out_queue.events()[i] {
                Some((_, e)) => match e {
                    Events::InputPoll(poll) => {
                        self.client_socket
                            .send_to(&[0; 10], self.server_socket.local_addr().unwrap())
                            .unwrap();
                        //TODO: replace above with send socket
                        send_socket(&mut self.client_socket)?;
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
            let server_msg = try_peek_socket(&mut self.client_socket)?;
            let client_msgs = try_peek_socket(&mut self.server_socket)?;
        }

        Ok(())
    }
}

fn send_socket(socket: &mut UdpSocket) -> Result<(), String> {
    Ok(())
}

fn try_peek_socket(socket: &mut UdpSocket) -> Result<Option<Packet>, String> {
    let mut buf = [0; PACKET_BYTE_LENGTH_BYTE_SIZE];
    match socket.peek(&mut buf) {
        Ok(received) => {
            println!("received {} bytes", received);
            // Read data
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

use crate::event_queue;
use crate::lib_core::{encryption::Crc32, time::Timer, LookUpGod};
use event_queue::*;
use std::convert::TryInto;
use std::net::UdpSocket;
use std::time::Duration;
const CHECKSUM_BYTE_LEN: usize = Crc32::CHECKSUM_BYTE_LEN;
const PACKET_BYTE_LENGTH_BYTE_SIZE: usize = 420; // BLAZE IT (in actuality, going off of http://ithare.com/64-network-dos-and-donts-for-game-engines-part-v-udp/ to limit the size to under 500 for MTU purposes)
pub struct Packet {
    data: [u8; PACKET_BYTE_LENGTH_BYTE_SIZE],
}
impl Packet {
    pub const TOTAL_PACKET_LEN: usize = CHECKSUM_BYTE_LEN + PACKET_BYTE_LENGTH_BYTE_SIZE;

    pub fn new() -> Self {
        Self {
            data: [0; PACKET_BYTE_LENGTH_BYTE_SIZE],
        }
    }

    pub fn from_bytes(lug: &LookUpGod, bytes: [u8; Self::TOTAL_PACKET_LEN]) -> Option<Self> {
        match decrypt_data(lug, bytes) {
            Some(data) => Some(Self { data: data }),
            None => None,
        }
    }

    pub fn to_network_bytes(&self, lug: &LookUpGod) -> [u8; Self::TOTAL_PACKET_LEN] {
        hash_data(lug, self.data)
    }
}

fn hash_data(
    lug: &LookUpGod,
    data: [u8; PACKET_BYTE_LENGTH_BYTE_SIZE],
) -> [u8; PACKET_BYTE_LENGTH_BYTE_SIZE + CHECKSUM_BYTE_LEN] {
    let checksum = lug.crc32.hash(data.iter());
    let mut result = [0; PACKET_BYTE_LENGTH_BYTE_SIZE + CHECKSUM_BYTE_LEN];
    for (ref mut to, from) in result.iter().zip(checksum.iter().chain(data.iter())) {
        *to = from
    }
    // For some reason checksum wasn't getting copied. Just decided to roll with the explicit setting.
    result[0] = checksum[0];
    result[1] = checksum[1];
    result[2] = checksum[2];
    result[3] = checksum[3];
    return result;
}

#[allow(unused_mut)]
fn decrypt_data(
    lug: &LookUpGod,
    data: [u8; PACKET_BYTE_LENGTH_BYTE_SIZE + CHECKSUM_BYTE_LEN],
) -> Option<[u8; PACKET_BYTE_LENGTH_BYTE_SIZE]> {
    // if checksum matches, return packet otherwise None
    let packet_checksum = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    let data = &data[Crc32::CHECKSUM_BYTE_LEN..];

    let calculated_checksum = u32::from_le_bytes(lug.crc32.hash(data.iter()));

    if packet_checksum == calculated_checksum {
        let mut result = [0; PACKET_BYTE_LENGTH_BYTE_SIZE];
        for (ref mut to, from) in result.iter().zip(data.iter()) {
            *to = from
        }
        return Some(result);
    }

    None
}

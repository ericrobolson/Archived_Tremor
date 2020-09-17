use crate::lib_core::{encryption::Crc32, LookUpGod};
use std::fmt::Debug;

pub type Sequence = u16;
pub const MAX_SEQUENCE_VALUE: Sequence = Sequence::MAX;
pub const ACK_BIT_LENGTH: usize = 32; // the number of bits in the 'ack_bits' property of the packet
const ACK_BITS_BYTE_LEN: usize = ACK_BIT_LENGTH / 8;
const SEQUENCE_BYTE_LEN: usize = 16 / 8;
const ACK_BYTE_LEN: usize = SEQUENCE_BYTE_LEN;
const ACK_HEADER_BYTE_LEN: usize = SEQUENCE_BYTE_LEN + ACK_BYTE_LEN + ACK_BITS_BYTE_LEN; // 2 for Sequence, 2 for ack, 4 for ack_bits

const CHECKSUM_BYTE_LEN: usize = Crc32::CHECKSUM_BYTE_LEN;
const PACKET_DATA_BYTE_SIZE: usize = 420; // BLAZE IT (in actuality, going off of http://ithare.com/64-network-dos-and-donts-for-game-engines-part-v-udp/ to limit the size to under 500 for MTU purposes)

// Inspired by https://gafferongames.com/post/reliable_ordered_messages/

#[derive(Copy, Clone)]
pub struct Packet {
    /// The id of the packet/packet Sequence number
    sequence: Sequence,
    // The most recent packet Sequence number recieved
    ack: Sequence,

    /// The previous messages to ack. If bit n is set, then ack - n is acked
    ack_bits: u32,

    data: [u8; PACKET_DATA_BYTE_SIZE],
}

impl Packet {
    pub const TOTAL_PACKET_LEN: usize =
        CHECKSUM_BYTE_LEN + ACK_HEADER_BYTE_LEN + PACKET_DATA_BYTE_SIZE;

    pub fn new() -> Self {
        Self {
            sequence: 0,
            ack: 0,
            ack_bits: 0,
            data: [0; PACKET_DATA_BYTE_SIZE],
        }
    }

    pub fn set_sequence(&mut self, sequence: Sequence) {
        self.sequence = sequence;
    }

    pub fn sequence(&self) -> Sequence {
        self.sequence
    }

    pub fn set_ack(&mut self, ack: Sequence) {
        self.ack = ack;
    }

    pub fn ack(&self) -> Sequence {
        self.ack
    }

    pub fn set_ack_bits(&mut self, ack_bits: u32) {
        self.ack_bits = ack_bits;
    }

    // If bit n is set, then that means Sequence - n has been acked
    pub fn ack_bits(&self) -> u32 {
        self.ack_bits
    }

    #[allow(unused_mut)]
    pub fn from_bytes(lug: &LookUpGod, bytes: [u8; Self::TOTAL_PACKET_LEN]) -> Option<Self> {
        match decrypt_data(lug, bytes) {
            Some(data) => {
                //decrypt the ack + Sequence
                let sequence = Sequence::from_le_bytes([data[0], data[1]]);
                let ack = Sequence::from_le_bytes([data[2], data[3]]);
                let ack_bits = u32::from_ne_bytes([data[4], data[5], data[6], data[7]]); //NOTE: we use the native endian here as this is for bitshifting

                // Get the actual data
                let mut result = [0; PACKET_DATA_BYTE_SIZE];
                for (ref mut to, from) in result.iter().zip(data[8..].iter()) {
                    *to = from
                }

                return Some(Self {
                    sequence: sequence,
                    ack: ack,
                    ack_bits: ack_bits,
                    data: result,
                });
            }
            None => None,
        }
    }

    #[allow(unused_mut)]
    pub fn to_network_bytes(&self, lug: &LookUpGod) -> [u8; Self::TOTAL_PACKET_LEN] {
        // Combine acks + data
        let sequence = self.sequence.to_le_bytes();
        let ack = self.ack.to_le_bytes();
        let acks = self.ack_bits.to_ne_bytes(); //NOTE: we use the native endian here as this is for bitshifting

        let mut result = [0; ACK_HEADER_BYTE_LEN + PACKET_DATA_BYTE_SIZE];
        for (ref mut to, from) in result.iter().zip(
            sequence
                .iter()
                .chain(ack.iter().chain(acks.iter().chain(self.data.iter()))),
        ) {
            *to = from
        }

        hash_data(lug, result)
    }
}

impl Debug for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "{{ sequence:{}, ack:{}, ack_bits:{}, data:{:?} }}",
            self.sequence,
            self.ack,
            self.ack_bits,
            self.data.to_vec()
        )
    }
}

fn hash_data(
    lug: &LookUpGod,
    data: [u8; ACK_HEADER_BYTE_LEN + PACKET_DATA_BYTE_SIZE],
) -> [u8; Packet::TOTAL_PACKET_LEN] {
    let checksum = lug.crc32.hash(data.iter());
    let mut result = [0; Packet::TOTAL_PACKET_LEN];
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
    data: [u8; Packet::TOTAL_PACKET_LEN],
) -> Option<[u8; ACK_HEADER_BYTE_LEN + PACKET_DATA_BYTE_SIZE]> {
    // if checksum matches, return packet otherwise None
    let packet_checksum = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    let data = &data[Crc32::CHECKSUM_BYTE_LEN..];

    let calculated_checksum = u32::from_le_bytes(lug.crc32.hash(data.iter()));

    if packet_checksum == calculated_checksum {
        let mut result = [0; ACK_HEADER_BYTE_LEN + PACKET_DATA_BYTE_SIZE];
        for (ref mut to, from) in result.iter().zip(data.iter()) {
            *to = from
        }
        return Some(result);
    }

    None
}

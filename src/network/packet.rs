use crate::lib_core::{encryption::Crc32, LookUpGod};
use std::fmt::Debug;
use std::slice::Iter;

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

// not sure how to do this? maybe figure out a way to do traits for writing to a stream? that way you have consistent functionality across data types
/*
impl Iterator<u32> PacketStream<Iterator<u32>> {
    const BYTE_LEN: usize = 4;
}
*/

/// This is the core of the application. The ability to transmit data quickly and reliably.
/// Only pure data of 1's and 0's is transmitted.
#[derive(Copy, Clone)]
pub struct Packet {
    /// The id of the packet/packet Sequence number
    sequence: Sequence,
    // The most recent packet Sequence number recieved
    ack: Sequence,
    /// The previous messages to ack. If bit n is set, then ack - n is acked
    ack_bits: u32,
    data: [u8; PACKET_DATA_BYTE_SIZE],
    read_index: usize,
    write_index: usize,
}

const f32_bytes: usize = 4;

impl Packet {
    pub const TOTAL_PACKET_LEN: usize =
        CHECKSUM_BYTE_LEN + ACK_HEADER_BYTE_LEN + PACKET_DATA_BYTE_SIZE;

    pub fn new() -> Self {
        Self {
            sequence: 0,
            ack: 0,
            ack_bits: 0,
            data: [0; PACKET_DATA_BYTE_SIZE],
            read_index: 0,
            write_index: 0,
        }
    }

    pub fn write_f32(&mut self, f: f32) -> bool {
        let max_write_index = self.write_index + f32_bytes;
        if max_write_index < PACKET_DATA_BYTE_SIZE {
            let data = f.to_le_bytes();
            let mut j = 0;
            for i in self.write_index..max_write_index {
                self.data[i] = data[j];
                j += 1;
            }

            self.write_index += f32_bytes;

            return true;
        }

        false
    }

    pub fn read_f32(&mut self) -> Option<f32> {
        let max_read_index = self.read_index + f32_bytes;
        if max_read_index < PACKET_DATA_BYTE_SIZE {
            let data = [
                self.data[self.read_index + 0],
                self.data[self.read_index + 1],
                self.data[self.read_index + 2],
                self.data[self.read_index + 3],
            ];
            self.read_index += f32_bytes;

            return Some(f32::from_le_bytes(data));
        }

        None
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
                for (i, byte) in data[ACK_HEADER_BYTE_LEN..].iter().enumerate() {
                    result[i] = *byte;
                }

                return Some(Self {
                    sequence: sequence,
                    ack: ack,
                    ack_bits: ack_bits,
                    data: result,
                    read_index: 0,
                    write_index: 0,
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

        for (i, byte) in sequence
            .iter()
            .chain(ack.iter().chain(acks.iter().chain(self.data.iter())))
            .enumerate()
        {
            result[i] = *byte;
        }

        hash_data(lug, result)
    }
}

impl Debug for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "{{ sequence:{}, ack:'{}', ack_bits:'{}', data: '{:?}' }}",
            self.sequence,
            self.ack,
            self.ack_bits,
            String::from_utf8_lossy(&self.data.to_vec()).into_owned()
        )
    }
}

fn hash_data(
    lug: &LookUpGod,
    data: [u8; ACK_HEADER_BYTE_LEN + PACKET_DATA_BYTE_SIZE],
) -> [u8; Packet::TOTAL_PACKET_LEN] {
    let checksum = lug.crc32.hash(data.iter());
    let mut result = [0; Packet::TOTAL_PACKET_LEN];
    for (i, byte) in checksum.iter().chain(data.iter()).enumerate() {
        result[i] = *byte;
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
        for (i, byte) in data.iter().enumerate() {
            result[i] = *byte;
        }
        return Some(result);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet_serialize_serializes_sequence() {
        let lug = LookUpGod::new();

        let mut packet = Packet::new();
        packet.set_sequence(333);

        let bytes = packet.to_network_bytes(&lug);
        let seq = Sequence::from_le_bytes([bytes[4], bytes[5]]);

        assert_eq!(333, seq);
    }

    #[test]
    fn packet_serializes_and_deserializes_deterministically() {
        let lug = LookUpGod::new();

        let mut packet = Packet::new();
        packet.set_sequence(333);
        packet.set_ack_bits(959321);

        let f1 = 34.33;
        packet.write_f32(f1);

        let bytes = packet.to_network_bytes(&lug);
        let deserialized = Packet::from_bytes(&lug, bytes);
        assert_eq!(true, deserialized.is_some());
        let deserialized = deserialized.unwrap();

        assert_eq!(packet.sequence(), deserialized.sequence());
        assert_eq!(packet.ack_bits(), deserialized.ack_bits());
        assert_eq!(packet.data.to_vec(), deserialized.data.to_vec());
    }

    #[test]
    fn packet_write_read_f32_works_as_expected() {
        let lug = LookUpGod::new();

        let mut packet = Packet::new();
        packet.set_sequence(333);
        packet.set_ack_bits(959321);

        let f1 = 34.33;
        packet.write_f32(f1);

        let f2 = 34321.33245;
        packet.write_f32(f2);

        let read_f1 = packet.read_f32();
        assert_eq!(true, read_f1.is_some());
        let read_f1 = read_f1.unwrap();
        assert_eq!(f1, read_f1);

        let read_f2 = packet.read_f32();
        assert_eq!(true, read_f2.is_some());
        let read_f2 = read_f2.unwrap();
        assert_eq!(f2, read_f2);
    }
}

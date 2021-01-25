

lazy_static! {
    pub static ref CRC32: Crc32 = Crc32::new();
}


// Cross platform deterministic CRC-32 implementation. Primarily used for non critical checksums.
pub struct Crc32 {
    table: [u32; 256],
}

impl Crc32 {
    pub const CHECKSUM_BYTE_LEN: usize = 4;
    pub fn new() -> Self {
        //https://rosettacode.org/wiki/CRC-32#Rust

        let mut table = [0; 256];
        // Use little endian for cross platform checks
        for n in 0..256 {
            table[n as usize] = ((0..8).fold(n as u32, |acc, _| match acc & 1 {
                1 => 0xedb88320 ^ (acc >> 1),
                _ => acc >> 1,
            }))
            .to_le();
        }

        Self { table: table }
    }

    pub fn hash(&self, buf: std::slice::Iter<u8>) -> [u8; Self::CHECKSUM_BYTE_LEN] {
        // Use little endian for cross platform checks
        (!buf.fold(!0, |acc, octet| {
            (acc >> 8) ^ self.table[((acc & 0xff) ^ *octet as u32) as usize]
        }))
        .to_le_bytes()
    }
}

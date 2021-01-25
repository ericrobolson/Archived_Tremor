use super::Bitstream;

/// A type that may be packed or unpacked from a Bitstream. It is preferable to attempt serializations/deserializations through the Bitstream class as it will prevent overflows.
pub trait Packable {
    fn bit_size() -> usize;
    fn byte_len() -> usize;
    fn unpack(stream: &mut Bitstream) -> Self;
    fn pack(&self, stream: &mut Bitstream);
}

impl Packable for bool {
    fn bit_size() -> usize {
        1
    }

    fn byte_len() -> usize {
        1
    }

    fn unpack(stream: &mut Bitstream) -> Self {
        match stream.read_byte(Self::bit_size()) {
            1 => true,
            _ => false,
        }
    }

    fn pack(&self, stream: &mut Bitstream) {
        let bytes = match self {
            true => 1,
            false => 0,
        };

        stream.write_byte(bytes, Self::bit_size());
    }
}

impl Packable for u32 {
    fn bit_size() -> usize {
        32
    }

    fn byte_len() -> usize {
        std::mem::size_of::<u32>()
    }

    fn unpack(stream: &mut Bitstream) -> Self {
        let mut bytes = [0; 4];
        let byte_len = Self::byte_len();
        for i in 0..byte_len {
            bytes[i] = stream.read_byte(8);
        }

        Self::from_le_bytes(bytes)
    }

    fn pack(&self, stream: &mut Bitstream) {
        let bytes = self.to_le_bytes();

        for i in 0..Self::byte_len() {
            stream.write_byte(bytes[i], 8);
        }
    }
}

impl Packable for f32 {
    fn bit_size() -> usize {
        32
    }

    fn byte_len() -> usize {
        std::mem::size_of::<f32>()
    }

    fn unpack(stream: &mut Bitstream) -> Self {
        let mut bytes = [0; 4];
        let byte_len = Self::byte_len();
        for i in 0..byte_len {
            bytes[i] = stream.read_byte(8);
        }

        Self::from_le_bytes(bytes)
    }

    fn pack(&self, stream: &mut Bitstream) {
        let bytes = self.to_le_bytes();

        for i in 0..Self::byte_len() {
            stream.write_byte(bytes[i], 8);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packable_bools() {
        let mut bs = Bitstream::new(100);

        bs.write(true);
        bs.write(false);
        bs.write(true);
        bs.write(true);

        bs.flush();

        assert_eq!(true, bs.read().unwrap());
        assert_eq!(false, bs.read().unwrap());
        assert_eq!(true, bs.read().unwrap());
        assert_eq!(true, bs.read().unwrap());
    }

    #[test]
    fn packable_f32() {
        let mut bs = Bitstream::new(100);

        bs.write(0.1);
        bs.write(0.2);
        bs.write(-999.9);
        bs.write(201094.1);

        bs.flush();

        assert_eq!(0.1, bs.read::<f32>().unwrap());
        assert_eq!(0.2, bs.read::<f32>().unwrap());
        assert_eq!(-999.9, bs.read::<f32>().unwrap());
        assert_eq!(201094.1, bs.read::<f32>().unwrap());
    }

    #[test]
    fn packable_u32() {
        let mut bs = Bitstream::new(100);

        bs.write(343);
        bs.write(223);
        bs.write(3);
        bs.write(0);

        bs.flush();

        assert_eq!(343, bs.read::<u32>().unwrap());
        assert_eq!(223, bs.read::<u32>().unwrap());
        assert_eq!(3, bs.read::<u32>().unwrap());
        assert_eq!(0, bs.read::<u32>().unwrap());
    }
}

mod packable;

pub use packable::*;

const STAGING_BUFFER_LEN: usize = 16;

/// Bitstream manager class for compact serialization.
pub struct Bitstream {
    staging: u16,
    buffer: Vec<u8>,
    total_bit_len: usize,
    number_of_staging_bits: usize,
    max_bits: usize,
}

impl Bitstream {
    /// Creates a new Bitstream with a set number of bytes.
    pub fn new(byte_len: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(byte_len),
            staging: 0,
            total_bit_len: 0,
            number_of_staging_bits: 0,
            max_bits: byte_len * 8,
        }
    }

    /// Flushes all pending operations and returns the final Vec of bytes.
    pub fn buffer(&mut self) -> Vec<u8> {
        self.flush();

        let buffer = self.buffer.clone();

        self.total_bit_len = 0;
        self.number_of_staging_bits = 0;
        self.buffer.clear();

        buffer
    }

    /// Write the given value to the stream. Will return true if it succeeded, or false if it wouldn't.
    pub fn write<T>(&mut self, value: T) -> bool
    where
        T: Packable,
    {
        if self.can_write(T::bit_size()) {
            value.pack(self);
            true
        } else {
            false
        }
    }

    /// Attempt to deseralize the type from the stream. Will return Some(T) if it was successful.
    pub fn read<T>(&mut self) -> Option<T>
    where
        T: Packable,
    {
        if self.can_read(T::bit_size()) {
            Some(T::unpack(self))
        } else {
            None
        }
    }

    /// Flush the staging buffer to the actual buffer.
    fn flush(&mut self) {
        if self.number_of_staging_bits < 8 && self.number_of_staging_bits > 0 {
            let padding_amount = 8 - self.number_of_staging_bits;
            self.write_byte(0, padding_amount);
        }

        self.staging = 0;
        self.number_of_staging_bits = 0;
    }

    /// Check to see if the stream can write more bits.
    fn can_write(&self, bits: usize) -> bool {
        self.total_bit_len + bits <= self.max_bits
    }

    /// Check to see if the stream can read the given number of bits.
    fn can_read(&self, bits: usize) -> bool {
        match self.total_bit_len.checked_sub(bits) {
            Some(_) => true,
            None => false,
        }
    }

    /// Write the given bits.
    fn write_byte(&mut self, value: u8, bits: usize) {
        // Boundary check
        let bits = {
            if bits > 8 {
                8
            } else {
                bits
            }
        };

        self.staging = self.staging << bits;

        // Ensure that the value is only what was specified
        let value = {
            let shift = 8 - bits;
            (value << shift) >> shift
        };

        // Update the staging buffer + number of bits
        self.staging |= value as u16;
        self.number_of_staging_bits += bits;
        self.total_bit_len += bits;

        // Write data to buffer
        while self.number_of_staging_bits >= 8 {
            let b = (self.staging >> self.number_of_staging_bits - 8) as u8;

            self.buffer.push(b);
            self.number_of_staging_bits -= 8;

            let mask = (0b1111_1111_1111_1111 >> self.number_of_staging_bits)
                << self.number_of_staging_bits;

            self.staging = self.staging & !mask; // zero out shifted bits
        }
    }

    fn read_byte(&mut self, bits: usize) -> u8 {
        if bits == 0 || self.total_bit_len == 0 {
            return 0;
        }

        // Boundary check
        let bits = {
            if bits > 8 {
                8
            } else {
                bits
            }
        };

        // Move byte from buffer to staging, shifting existing values to the left to make space for the new byte
        if self.number_of_staging_bits < bits && self.buffer.len() > 0 {
            let b = self.buffer.remove(0) as u16;

            self.staging = (self.staging << 8) | b;

            self.number_of_staging_bits += 8;
        }

        // Read off staging buffer
        let value = {
            // rshift to get the value we want
            let v = self.staging >> self.number_of_staging_bits - bits;

            v as u8
        };

        // Remove the bits from the count
        self.number_of_staging_bits -= bits;
        self.total_bit_len -= bits;

        // Zero out leftmost bits in the staging buffer
        if self.number_of_staging_bits == 0 {
            self.staging = 0;
        } else {
            let shift = STAGING_BUFFER_LEN - self.number_of_staging_bits;
            let mask = (0b1111_1111_1111_1111 << shift) >> shift;

            self.staging = self.staging & mask;
        }

        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitstream_new() {
        let bs = Bitstream::new(100);

        assert_eq!(0, bs.staging);
        assert_eq!(0, bs.total_bit_len);
        assert_eq!(0, bs.number_of_staging_bits);
        assert_eq!(100 * 8, bs.max_bits);
    }

    #[test]
    fn bitstream_read_byte_in_staging_and_buffer_works_as_expected() {
        let byte_capacity = 2;
        let mut bs = Bitstream::new(byte_capacity);

        bs.write_byte(0b_0001_1111, 5);
        bs.write_byte(0b_0000_0000, 8);
        assert_eq!(5, bs.number_of_staging_bits);

        bs.flush();

        let b = bs.read_byte(5);
        assert_eq!(0b_0001_1111, b);

        let b = bs.read_byte(8);
        assert_eq!(0b_0000_0000, b);

        let mut bs = Bitstream::new(200);
        let v1 = 0b_0001_1111;
        let v1_len = 7;

        let v2 = 0b_0011_1101;
        let v2_len = 6;

        let v3 = 0b_0000_0110;
        let v3_len = 3;

        bs.write_byte(v1, v1_len);
        bs.write_byte(v2, v2_len);
        bs.write_byte(v3, v3_len);

        bs.flush();

        assert_eq!(v1, bs.read_byte(v1_len));
        assert_eq!(v2, bs.read_byte(v2_len));
        assert_eq!(v3, bs.read_byte(v3_len));
    }

    #[test]
    fn bitstream_read_byte_in_staging_buffer_works_as_expected() {
        let byte_capacity = 2;
        let mut bs = Bitstream::new(byte_capacity);

        bs.write_byte(0, 1);
        bs.write_byte(1, 1);
        bs.write_byte(0, 1);
        bs.write_byte(1, 1);

        let b = bs.read_byte(1);
        assert_eq!(0, b);
        assert_eq!(3, bs.total_bit_len);
        assert_eq!(3, bs.number_of_staging_bits);

        let b = bs.read_byte(1);
        assert_eq!(1, b);
        assert_eq!(2, bs.total_bit_len);
        assert_eq!(2, bs.number_of_staging_bits);

        let b = bs.read_byte(1);
        assert_eq!(0, b);
        assert_eq!(1, bs.total_bit_len);
        assert_eq!(1, bs.number_of_staging_bits);

        let b = bs.read_byte(1);
        assert_eq!(1, b);
        assert_eq!(0, bs.total_bit_len);
        assert_eq!(0, bs.number_of_staging_bits);

        bs.write_byte(1, 1);
        bs.write_byte(0, 1);
        bs.write_byte(0, 1);
        bs.write_byte(1, 1);

        let b = bs.read_byte(1);
        assert_eq!(1, b);
        assert_eq!(3, bs.total_bit_len);
        assert_eq!(3, bs.number_of_staging_bits);

        let b = bs.read_byte(1);
        assert_eq!(0, b);
        assert_eq!(2, bs.total_bit_len);
        assert_eq!(2, bs.number_of_staging_bits);

        let b = bs.read_byte(1);
        assert_eq!(0, b);
        assert_eq!(1, bs.total_bit_len);
        assert_eq!(1, bs.number_of_staging_bits);

        let b = bs.read_byte(1);
        assert_eq!(1, b);
        assert_eq!(0, bs.total_bit_len);
        assert_eq!(0, bs.number_of_staging_bits);

        bs.write_byte(1, 2);
        bs.write_byte(0, 1);
        bs.write_byte(0, 3);
        bs.write_byte(1, 1);

        let b = bs.read_byte(2);
        assert_eq!(1, b);
        assert_eq!(5, bs.total_bit_len);
        assert_eq!(5, bs.number_of_staging_bits);

        let b = bs.read_byte(1);
        assert_eq!(0, b);
        assert_eq!(4, bs.total_bit_len);
        assert_eq!(4, bs.number_of_staging_bits);

        let b = bs.read_byte(3);
        assert_eq!(0, b);
        assert_eq!(1, bs.total_bit_len);
        assert_eq!(1, bs.number_of_staging_bits);

        let b = bs.read_byte(1);
        assert_eq!(1, b);
        assert_eq!(0, bs.total_bit_len);
        assert_eq!(0, bs.number_of_staging_bits);
    }

    #[test]
    fn bitstream_read_byte_0bits_returns0() {
        let byte_capacity = 2;
        let mut bs = Bitstream::new(byte_capacity);

        let byte = bs.read_byte(0);
        assert_eq!(0b_0000_0000_0000_0000, byte);

        let byte = bs.read_byte(8);
        assert_eq!(0b_0000_0000_0000_0000, byte);
    }

    #[test]
    fn bitstream_write_byte_under8_sets_staging() {
        let byte_capacity = 2;
        let mut bs = Bitstream::new(byte_capacity);

        bs.write_byte(1, 1);
        assert_eq!(0b_0000_0000_0000_0001, bs.staging);
        assert_eq!(true, bs.buffer.is_empty());
        assert_eq!(1, bs.total_bit_len);
        assert_eq!(1, bs.number_of_staging_bits);

        bs.write_byte(0, 1);
        assert_eq!(0b_0000_0000_0000_0010, bs.staging);
        assert_eq!(true, bs.buffer.is_empty());
        assert_eq!(2, bs.total_bit_len);
        assert_eq!(2, bs.number_of_staging_bits);

        bs.write_byte(1, 2);
        assert_eq!(0b_0000_0000_0000_1001, bs.staging);
        assert_eq!(true, bs.buffer.is_empty());
        assert_eq!(4, bs.total_bit_len);
        assert_eq!(4, bs.number_of_staging_bits);

        bs.write_byte(1, 2);
        assert_eq!(0b_0000_0000_0010_0101, bs.staging);
        assert_eq!(true, bs.buffer.is_empty());
        assert_eq!(6, bs.total_bit_len);
        assert_eq!(6, bs.number_of_staging_bits);

        bs.write_byte(0, 1);
        assert_eq!(0b_0000_0000_0100_1010, bs.staging);
        assert_eq!(true, bs.buffer.is_empty());
        assert_eq!(7, bs.total_bit_len);
        assert_eq!(7, bs.number_of_staging_bits);
    }

    #[test]
    fn bitstream_write_byte_over8_sets_staging_and_copies_to_buffer() {
        let byte_capacity = 2;
        let max_bits = byte_capacity * 8;
        let mut bs = Bitstream::new(byte_capacity);

        // Set it up to store 7 bits
        bs.write_byte(0b_0111_1111, 7);
        assert_eq!(0b_0000_0000_0111_1111, bs.staging);
        assert_eq!(true, bs.buffer.is_empty());
        assert_eq!(7, bs.total_bit_len);
        assert_eq!(7, bs.number_of_staging_bits);

        // add 1 to move it all to buffer
        bs.write_byte(1, 1);
        assert_eq!(0, bs.staging);
        assert_eq!(false, bs.buffer.is_empty());
        assert_eq!(vec![255], bs.buffer);
        assert_eq!(8, bs.total_bit_len);
        assert_eq!(0, bs.number_of_staging_bits);

        // Set it up to store 7 bits
        bs.write_byte(0b_0111_1111, 7);
        assert_eq!(0b_0000_0000_0111_1111, bs.staging);
        assert_eq!(false, bs.buffer.is_empty());
        assert_eq!(vec![255], bs.buffer);
        assert_eq!(15, bs.total_bit_len);
        assert_eq!(7, bs.number_of_staging_bits);

        // add 2 to move all but 1 to buffer
        bs.write_byte(0b_1111_1111, 2);
        assert_eq!(1, bs.staging);
        assert_eq!(false, bs.buffer.is_empty());
        assert_eq!(vec![255, 255], bs.buffer);
        assert_eq!(17, bs.total_bit_len);
        assert_eq!(1, bs.number_of_staging_bits);

        // check to make sure bits over 8 get set to 8
        let byte_capacity = 2;
        let mut bs = Bitstream::new(byte_capacity);
        bs.write_byte(0b_1111_1110, 1);
        bs.write_byte(0b_1011_1011, 9);
        assert_eq!(1, bs.staging);
        assert_eq!(false, bs.buffer.is_empty());
        assert_eq!(vec![93], bs.buffer);
        assert_eq!(9, bs.total_bit_len);
        assert_eq!(1, bs.number_of_staging_bits);
    }

    #[test]
    fn bitstream_can_write() {
        let byte_capacity = 2;
        let max_bits = byte_capacity * 8;
        let mut bs = Bitstream::new(byte_capacity);
        assert_eq!(true, bs.can_write(0));
        assert_eq!(true, bs.can_write(1));
        assert_eq!(true, bs.can_write(max_bits - 1));
        assert_eq!(true, bs.can_write(max_bits));

        assert_eq!(false, bs.can_write(max_bits + 1));
        bs.total_bit_len += 1;

        assert_eq!(true, bs.can_write(0));
        assert_eq!(true, bs.can_write(1));
        assert_eq!(true, bs.can_write(max_bits - 1));
        assert_eq!(false, bs.can_write(max_bits));
    }

    #[test]
    fn bitstream_can_read() {
        let byte_capacity = 2;
        let max_bits = byte_capacity * 8;
        let mut bs = Bitstream::new(byte_capacity);
        assert_eq!(true, bs.can_read(0));
        assert_eq!(false, bs.can_read(1));

        bs.total_bit_len += 1;
        assert_eq!(true, bs.can_read(1));
        assert_eq!(false, bs.can_read(2));
    }
}

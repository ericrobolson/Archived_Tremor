pub fn serialize_f32(f: f32) -> [u8; 4] {
    return f.to_le_bytes();
}

pub fn deserialize_f32(bytes: [u8; 4]) -> f32 {
    return f32::from_le_bytes(bytes);
}

pub fn serialize_i32(i: i32) -> [u8; 4] {
    return i.to_le_bytes();
}

pub fn deserialize_i32(bytes: [u8; 4]) -> i32 {
    return i32::from_le_bytes(bytes);
}

pub fn new_bit_array() -> u32 {
    0
}

pub fn push_bit(bytes: u32, bit: bool) -> u32 {
    // lshift
    let mut shift = bytes << 1;
    // add final bit
    if bit {
        shift = shift | 1;
    }

    shift
}

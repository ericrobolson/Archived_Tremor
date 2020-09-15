fn serialize_f32(f: f32) -> [u8; 4] {
    return f.to_le_bytes();
}

fn deserialize_f32(bytes: [u8; 4]) -> f32 {
    return f32::from_le_bytes(bytes);
}

fn serialize_i32(i: i32) -> [u8; 4] {
    return i.to_le_bytes();
}

fn deserialize_i32(bytes: [u8; 4]) -> i32 {
    return i32::from_le_bytes(bytes);
}

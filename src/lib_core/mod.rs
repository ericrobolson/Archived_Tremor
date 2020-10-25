pub mod data_structures;
pub mod ecs;
pub mod encryption;
pub mod input;
pub mod math;
pub mod serialization;
pub mod shapes;
pub mod time;
pub mod voxels;

/// Class used for caching lookup tables, hash functions, etc.
pub struct LookUpGod {
    pub crc32: encryption::Crc32,
}

impl LookUpGod {
    pub fn new() -> Self {
        Self {
            crc32: encryption::Crc32::new(),
        }
    }
}

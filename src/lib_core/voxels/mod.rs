mod chunk_manager;
pub use chunk_manager::*;

mod chunk;
pub use chunk::*;

pub mod rle_chunk;

pub struct Palette {}
pub type PaletteIndex = u8;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Voxel {
    Empty,
    Skin,
    Bone,
    Cloth,
    Metal,
}

pub trait VoxelNumeric
where
    Self: std::marker::Sized,
{
    fn is_empty(&self) -> bool;
    fn distance_field(&self) -> Self;
    fn set_values(&mut self, value: u8);
    fn values(&self) -> Self;
    fn set_distance_field(&mut self, distance: u8);
    fn set_voxel(&mut self, voxel: Voxel);
    fn voxel(&self) -> Voxel;
    fn serialize(&self) -> [u8; 1];
    fn deserialize(byte: [u8; 1]) -> Self;
}

impl VoxelNumeric for u8 {
    fn is_empty(&self) -> bool {
        self & 1 == 0
    }
    fn set_distance_field(&mut self, distance: u8) {
        if !self.is_empty() {
            return;
        }
        self.set_values(distance);
    }
    fn distance_field(&self) -> Self {
        if self.is_empty() == false {
            return 0;
        }

        self.values()
    }
    fn set_values(&mut self, value: u8) {
        let value = value << 1;
        *self &= value;
    }
    fn values(&self) -> Self {
        self >> 1
    }
    fn set_voxel(&mut self, voxel: Voxel) {
        if voxel == Voxel::Empty {
            *self = 0;
        } else {
            // Get voxel value, shift over and set active
            let value: u8 = voxel.into();
            let value = (value << 1) | 1;

            *self = value;
        }
    }
    fn voxel(&self) -> Voxel {
        if self.is_empty() {
            return Voxel::Empty;
        }

        self.values().into()
    }
    fn serialize(&self) -> [u8; 1] {
        self.to_le_bytes()
    }
    fn deserialize(bytes: [u8; 1]) -> Self {
        u8::from_le_bytes(bytes)
    }
}

impl Into<Voxel> for u8 {
    fn into(self) -> Voxel {
        match self {
            0 => Voxel::Empty,
            1 => Voxel::Skin,
            2 => Voxel::Bone,
            3 => Voxel::Cloth,
            4 => Voxel::Metal,
            _ => Voxel::Empty,
        }
    }
}

impl Into<u8> for Voxel {
    fn into(self) -> u8 {
        match self {
            Voxel::Empty => 0,
            Voxel::Skin => 1,
            Voxel::Bone => 2,
            Voxel::Cloth => 3,
            Voxel::Metal => 4,
        }
    }
}

impl Into<u8> for &Voxel {
    fn into(self) -> u8 {
        match self {
            Voxel::Empty => 0,
            Voxel::Skin => 1,
            Voxel::Bone => 2,
            Voxel::Cloth => 3,
            Voxel::Metal => 4,
        }
    }
}

impl Voxel {
    pub fn to_color(&self) -> (f32, f32, f32) {
        match self {
            Voxel::Empty => color(0, 0, 0),
            Voxel::Skin => color(252, 215, 172),
            Voxel::Bone => color(255, 244, 232),
            Voxel::Cloth => color(41, 216, 255),
            Voxel::Metal => color(83, 94, 97),
        }
    }
}

fn color(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = (r as f32) / 255.0;
    let g = (g as f32) / 255.0;
    let b = (b as f32) / 255.0;

    (r, g, b)
}

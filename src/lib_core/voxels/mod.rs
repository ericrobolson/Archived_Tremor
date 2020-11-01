mod chunk_manager;
pub use chunk_manager::*;

mod chunk;
pub use chunk::*;

mod palette;
pub mod rle_chunk;
pub use palette::{Color, Palette, PaletteIndex};

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
        let is_empty = *self & 1;
        let value = value | is_empty;
        *self = value;
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
    pub fn to_color(&self) -> (u8, u8, u8) {
        match self {
            Voxel::Empty => (0, 0, 0),
            Voxel::Skin => (252, 215, 172),
            Voxel::Bone => (255, 244, 232),
            Voxel::Cloth => (41, 216, 255),
            Voxel::Metal => (83, 94, 97),
        }
    }

    pub fn palette_index(&self) -> u8 {
        self.into()
    }
}

fn color(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = (r as f32) / 255.0;
    let g = (g as f32) / 255.0;
    let b = (b as f32) / 255.0;

    (r, g, b)
}

#[cfg(test)]
mod tests {
    use super::VoxelNumeric;
    use super::*;

    #[test]
    fn VoxelNumeric_u8_IsEmpty_WorksAsExpected() {
        let value: u8 = 0;
        assert_eq!(true, value.is_empty());

        let value: u8 = 0b1111_1110;
        assert_eq!(true, value.is_empty());

        let value: u8 = 1;
        assert_eq!(false, value.is_empty());

        let value: u8 = 0b1111_1111;
        assert_eq!(false, value.is_empty());
    }

    #[test]
    fn VoxelNumeric_u8_SetVoxel_WorksAsExpected() {
        let mut value: u8 = 0b1111_1111;
        value.set_voxel(Voxel::Empty);
        assert_eq!(0, value);

        let mut value: u8 = 0;
        value.set_voxel(Voxel::Bone);
        let expected: u8 = Voxel::Bone.into();
        assert_eq!(expected << 1 | 1, value);

        value.set_voxel(Voxel::Cloth);
        let expected: u8 = Voxel::Cloth.into();
        assert_eq!(expected << 1 | 1, value);
    }

    #[test]
    fn VoxelNumeric_u8_GetVoxel_WorksAsExpected() {
        let mut value: u8 = 0b1111_1111;
        let voxel = Voxel::Empty;
        value.set_voxel(voxel);
        assert_eq!(voxel, value.voxel());

        let voxel = Voxel::Bone;
        value.set_voxel(voxel);
        assert_eq!(voxel, value.voxel());

        let voxel = Voxel::Skin;
        value.set_voxel(voxel);
        assert_eq!(voxel, value.voxel());
    }

    #[test]
    fn VoxelNumeric_u8_Values_WorksAsExpected() {
        let mut value: u8 = 0b1111_1111;
        let expected: u8 = 0b0111_1111;
        assert_eq!(expected, value.values());

        let mut value: u8 = 0b0101_0101;
        let expected: u8 = 0b0010_1010;
        assert_eq!(expected, value.values());
    }

    #[test]
    fn VoxelNumeric_u8_set_values_WorksAsExpected() {
        let mut value: u8 = 0;
        let expected: u8 = 0b1111_1110;
        value.set_values(0b0111_1111);
        assert_eq!(expected, value);

        let mut value: u8 = 1;
        let expected: u8 = 0b1111_1111;
        value.set_values(0b0111_1111);
        assert_eq!(expected, value);
    }

    #[test]
    fn VoxelNumeric_u8_set_distance_field_WorksAsExpected() {
        let mut value: u8 = 0;
        let expected: u8 = 0b1111_1110;
        value.set_distance_field(0b0111_1111);
        assert_eq!(expected, value);

        let mut value: u8 = 1;
        let expected: u8 = 1;
        value.set_distance_field(0b0111_1111);
        assert_eq!(expected, value);
    }
}

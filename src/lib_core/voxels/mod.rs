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

impl Voxel {
    pub fn to_u8(&self) -> u8 {
        match self {
            Voxel::Empty => 0,
            Voxel::Skin => 1,
            Voxel::Bone => 2,
            Voxel::Cloth => 3,
            Voxel::Metal => 4,
        }
    }

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

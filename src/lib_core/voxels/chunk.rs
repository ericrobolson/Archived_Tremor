use crate::lib_core::{math::index_1d, math::index_3d, time::GameFrame};

use super::{Palette, PaletteIndex, Voxel};

pub struct Chunk {
    x_depth: usize,
    y_depth: usize,
    z_depth: usize,
    last_update: GameFrame,
    current_frame: GameFrame,
    voxels: Vec<Voxel>,
    colors: Vec<PaletteIndex>,
}

impl Chunk {
    pub fn new(x_depth: usize, y_depth: usize, z_depth: usize) -> Self {
        let capacity = x_depth * y_depth * z_depth;
        let mut voxels = Vec::with_capacity(capacity);
        let mut colors = Vec::with_capacity(capacity);

        for i in 0..capacity {
            // Always assign a voxel
            voxels.push(Voxel::Empty);
            colors.push(0); // TODO:
        }

        let mut i = 0;
        for x in 0..x_depth {
            for y in 0..y_depth {
                for z in 0..z_depth {
                    if x % 2 == 0 && y % 2 == 0 && z % 2 == 0 {
                        voxels[i] = Voxel::Bone;
                    } else if x % 3 == 1 && y % 3 == 1 && z % 3 == 1 {
                        voxels[i] = Voxel::Metal;
                    } else if x % 7 == 1 {
                        voxels[i] = Voxel::Cloth;
                    }

                    i += 1;
                }
            }
        }

        Self {
            x_depth,
            y_depth,
            z_depth,
            voxels,
            colors,
            last_update: 0,
            current_frame: 0,
        }
    }

    pub fn last_update(&self) -> GameFrame {
        self.last_update
    }

    pub fn update(&mut self, frame: GameFrame) {
        self.current_frame = frame;
    }

    pub fn capacity(&self) -> (usize, usize, usize) {
        (self.x_depth, self.y_depth, self.z_depth)
    }

    fn index_1d(&self, x: usize, y: usize, z: usize) -> usize {
        index_1d(x, y, z, self.x_depth, self.y_depth, self.z_depth)
    }

    fn index_3d(&self, i: usize) -> (usize, usize, usize) {
        index_3d(i, self.x_depth, self.y_depth, self.z_depth)
    }

    pub fn voxel(&self, x: usize, y: usize, z: usize) -> Voxel {
        return self.voxels[self.index_1d(x, y, z)];
    }

    pub fn set_voxel(&mut self, x: usize, y: usize, z: usize, voxel: Voxel) {
        let i = self.index_1d(x, y, z);
        self.voxels[i] = voxel;

        self.last_update = self.current_frame;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn VoxelChunk_New_DefaultsToEmpty() {
        let x_depth = 3;
        let y_depth = 4;
        let z_depth = 5;

        let chunk = Chunk::new(x_depth, y_depth, z_depth);

        assert_eq!(x_depth, chunk.x_depth);
        assert_eq!(y_depth, chunk.y_depth);
        assert_eq!(z_depth, chunk.z_depth);

        assert_eq!(x_depth * y_depth * z_depth, chunk.voxels.len());

        for voxel in chunk.voxels {
            assert_eq!(voxel, Voxel::Empty);
        }
    }

    #[test]
    fn VoxelChunk_index_1d_works_as_expected() {
        let x_depth = 3;
        let y_depth = 4;
        let z_depth = 5;

        let chunk = Chunk::new(x_depth, y_depth, z_depth);

        let (x, y, z) = (0, 0, 0);
        let expected = 0;
        let actual = chunk.index_1d(x, y, z);
        assert_eq!(expected, actual);

        let (x, y, z) = (1, 2, 3);
        let expected = x + y * x_depth + z * x_depth * y_depth;
        let actual = chunk.index_1d(x, y, z);
        assert_eq!(expected, actual);

        // Boundary check
        let (x, y, z) = (x_depth, y_depth, z_depth);
        let expected = 0;
        let actual = chunk.index_1d(x, y, z);
        assert_eq!(expected, actual);
    }

    #[test]
    fn VoxelChunk_index_3d_works_as_expected() {
        let x_depth = 3;
        let y_depth = 4;
        let z_depth = 5;

        let chunk = Chunk::new(x_depth, y_depth, z_depth);

        let (x, y, z) = (1, 2, 3);
        let expected = (x, y, z);
        let i = chunk.index_1d(x, y, z);
        let actual = chunk.index_3d(i);
        assert_eq!(expected, actual);

        let (x, y, z) = (3, 3, 2);
        let expected = (0, y, z);
        let i = chunk.index_1d(x, y, z);
        let actual = chunk.index_3d(i);
        assert_eq!(expected, actual);
    }

    #[test]
    fn VoxelChunk_set_voxels_works_as_expected() {
        let x_depth = 3;
        let y_depth = 4;
        let z_depth = 5;

        let mut chunk = Chunk::new(x_depth, y_depth, z_depth);

        let voxel = Voxel::Cloth;
        let (x, y, z) = (0, 0, 0);
        chunk.set_voxel(x, y, z, voxel);
        assert_eq!(voxel, chunk.voxel(x, y, z));

        let voxel = Voxel::Bone;
        let (x, y, z) = (0, 0, 0);
        chunk.set_voxel(x, y, z, voxel);
        assert_eq!(voxel, chunk.voxel(x, y, z));

        let voxel = Voxel::Bone;
        let (x, y, z) = (2, 3, 1);
        chunk.set_voxel(x, y, z, voxel);
        assert_eq!(voxel, chunk.voxel(x, y, z));

        let voxel = Voxel::Bone;
        let (x, y, z) = (3, 2, 4);
        chunk.set_voxel(x, y, z, voxel);
        assert_eq!(voxel, chunk.voxel(x, y, z));
    }
}

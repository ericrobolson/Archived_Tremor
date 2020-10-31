use crate::lib_core::{math::index_1d, math::index_2d_to_1d, math::index_3d, time::GameFrame};

use super::{Palette, PaletteIndex, Voxel};

/// Run length encoded voxels in the z direction
pub struct RleVoxels {
    voxels: Vec<(Voxel, u8)>,
}

impl RleVoxels {
    pub fn new(z_depth: usize) -> Self {
        let mut voxels = Vec::with_capacity(z_depth);
        voxels.push((Voxel::Empty, z_depth as u8));

        Self { voxels }
    }

    pub fn len(&self) -> usize {
        self.voxels.len()
    }

    pub fn voxel(&self, z: u8) -> Voxel {
        let mut count = 0;
        for (voxel, num_voxels) in &self.voxels {
            count += num_voxels;

            if z <= count {
                return *voxel;
            }
        }

        Voxel::Empty
    }

    pub fn set_voxel(&mut self, voxel: Voxel, z: u8) {
        // TODO: set voxel
        let mut count = 0;
        for (voxel, num_voxels) in &self.voxels {
            count += num_voxels;

            if z <= count {
                //  return *voxel;
            }
        }

        self.consolidate();
        unimplemented!();
    }

    // Join up duplicate voxels, reducing the size.
    fn consolidate(&mut self) {
        // TODO: test this
        let mut i = 0;
        loop {
            let j = i + 1;

            let len = self.len();
            if len < 2 || i == len || len <= j {
                return;
            }

            let (voxel_a, voxel_a_len) = self.voxels[i];
            let (voxel_b, voxel_b_len) = self.voxels[j];

            // If voxels are the same, merge them.
            if voxel_a == voxel_b {
                self.voxels[i] = (voxel_a, voxel_a_len + voxel_b_len);
                self.voxels.remove(j);
            } else {
                // If not the same, increment to next encoding
                i += 1;
            }
        }
    }
}

pub struct RleChunk {
    x_depth: usize,
    y_depth: usize,
    z_depth: usize,
    last_update: GameFrame,
    current_frame: GameFrame,
    voxels: Vec<RleVoxels>,
}

impl RleChunk {
    pub fn new(x_depth: usize, y_depth: usize, z_depth: usize) -> Self {
        let capacity = x_depth * y_depth;
        let mut voxels = Vec::with_capacity(capacity);

        for _ in 0..capacity {
            // Always assign a voxel
            voxels.push(RleVoxels::new(z_depth));
        }

        let mut chunk = Self {
            x_depth,
            y_depth,
            z_depth,
            voxels,
            last_update: 0,
            current_frame: 0,
        };

        for z in 0..z_depth {
            for y in 0..y_depth {
                for x in 0..x_depth {
                    if x % 2 == 0 && y % 2 == 0 && z % 2 == 0 {
                        chunk.set_voxel(x, y, z, Voxel::Bone);
                    } else if x % 3 == 1 && y % 3 == 1 && z % 3 == 1 {
                        chunk.set_voxel(x, y, z, Voxel::Metal);
                    } else if x % 7 == 1 {
                        chunk.set_voxel(x, y, z, Voxel::Cloth);
                    }
                }
            }
        }

        chunk
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

    pub fn voxels(&self) -> &Vec<RleVoxels> {
        &self.voxels
    }

    pub fn voxel(&self, x: usize, y: usize, z: usize) -> Voxel {
        self.voxels[index_2d_to_1d(x, y, self.x_depth, self.y_depth)].voxel(z as u8)
    }

    pub fn set_voxel(&mut self, x: usize, y: usize, z: usize, voxel: Voxel) {
        self.voxels[index_2d_to_1d(x, y, self.x_depth, self.y_depth)].set_voxel(voxel, z as u8);

        self.last_update = self.current_frame;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn RleVoxels_Consolidate_Empty_DoesNothing() {
        let z_depth = 5;

        let mut rle = RleVoxels::new(z_depth);

        assert_eq!((Voxel::Empty, z_depth as u8), rle.voxels[0]);
        rle.consolidate();
        assert_eq!((Voxel::Empty, z_depth as u8), rle.voxels[0]);
    }

    #[test]
    fn RleVoxels_set_voxel_works_as_expected() {
        let z_depth = 5;
        let mut rle = RleVoxels::new(z_depth);
        assert_eq!((Voxel::Empty, z_depth as u8), rle.voxels[0]);
        rle.set_voxel(Voxel::Bone, 0);
        assert_eq!((Voxel::Bone, 1), rle.voxels[0]);
    }

    #[test]
    fn VoxelChunk_New_DefaultsToEmpty() {
        let x_depth = 3;
        let y_depth = 4;
        let z_depth = 5;

        let chunk = RleChunk::new(x_depth, y_depth, z_depth);

        assert_eq!(x_depth, chunk.x_depth);
        assert_eq!(y_depth, chunk.y_depth);
        assert_eq!(z_depth, chunk.z_depth);

        assert_eq!(x_depth * y_depth * z_depth, chunk.voxels.len());

        for voxel in chunk.voxels {
            for i in 0..voxel.len() {
                assert_eq!(voxel.voxel(i as u8), Voxel::Empty);
            }
        }
    }

    #[test]
    fn VoxelChunk_set_voxels_works_as_expected() {
        let x_depth = 3;
        let y_depth = 4;
        let z_depth = 5;

        let mut chunk = RleChunk::new(x_depth, y_depth, z_depth);

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

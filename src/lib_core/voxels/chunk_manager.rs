use crate::lib_core::{
    math::{index_1d, index_3d, FixedNumber},
    time::GameFrame,
};

use super::{Chunk, Voxel};

pub struct ChunkManager {
    x_depth: usize,
    y_depth: usize,
    z_depth: usize,
    // The maximum allowed steps to calculate distance fields for
    max_distance_field: usize,
    pub chunk_size: (usize, usize, usize),
    pub voxel_count: (usize, usize, usize),
    pub voxel_resolution: FixedNumber,
    pub last_update: GameFrame,
    current_frame: GameFrame,
    pub chunks: Vec<Chunk>,
}

impl ChunkManager {
    pub fn new(x_depth: usize, y_depth: usize, z_depth: usize) -> Self {
        let capacity = x_depth * y_depth * z_depth;

        let chunk_size = 8;
        let chunk_size = (chunk_size, chunk_size, chunk_size);

        let max_distance_field: usize = 2;

        let voxel_resolution = FixedNumber::fraction(2.into());

        let mut d = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            d.push(0);
        }

        use rayon::prelude::*;

        let chunks = d
            .par_iter()
            .map(|_| {
                Chunk::new(
                    chunk_size.0,
                    chunk_size.1,
                    chunk_size.2,
                    max_distance_field as u8,
                )
            })
            .collect();

        let voxel_count = (
            chunk_size.0 * x_depth,
            chunk_size.1 * y_depth,
            chunk_size.2 * z_depth,
        );

        let mut manager = Self {
            voxel_resolution,
            max_distance_field,
            x_depth,
            y_depth,
            z_depth,
            last_update: 0,
            current_frame: 0,
            chunks,
            chunk_size,
            voxel_count,
        };

        // Something wack is going on.

        for z in 0..voxel_count.2 {
            for x in 0..voxel_count.0 {
                let mut yindex = voxel_count.1 - 1;
                manager.set_voxel(x, yindex, z, Voxel::Bone);

                if x % 2 == 0 {
                    yindex -= 1;
                    manager.set_voxel(x, yindex, z, Voxel::Cloth);
                }

                if z % 3 == 0 {
                    yindex -= 1;
                    manager.set_voxel(x, yindex, z, Voxel::Metal);
                }

                let max_y = {
                    if x % 2 == 0 && z % 2 == 0 {
                        4
                    } else {
                        1
                    }
                };
                for yindex in 0..max_y {
                    manager.set_voxel(x, yindex, z, Voxel::Metal);
                }
            }
        }

        manager
    }

    pub fn last_update(&self) -> GameFrame {
        self.last_update
    }

    pub fn update_frame(&mut self, frame: GameFrame) {
        self.current_frame = frame;
        for chunk in self.chunks.iter_mut() {
            chunk.update(frame);
        }

        // Now randomly turn on voxels
        for i in 0..2 {
            let x = crate::lib_core::math::rng(self.voxel_count.0 as u32);
            let y = crate::lib_core::math::rng(self.voxel_count.1 as u32);
            let z = crate::lib_core::math::rng(self.voxel_count.2 as u32);

            let voxel = match crate::lib_core::math::rng(4) {
                0 => Voxel::Bone,
                1 => Voxel::Cloth,
                2 => Voxel::Metal,
                _ => Voxel::Skin,
            };

            self.set_voxel(x as usize, y as usize, z as usize, voxel);
        }
    }

    pub fn capacity(&self) -> (usize, usize, usize) {
        (self.x_depth, self.y_depth, self.z_depth)
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    fn index_1d(&self, x: usize, y: usize, z: usize) -> usize {
        index_1d(x, y, z, self.x_depth, self.y_depth, self.z_depth)
    }

    fn index_3d(&self, i: usize) -> (usize, usize, usize) {
        index_3d(i, self.x_depth, self.y_depth, self.z_depth)
    }

    pub fn voxel(&self, x: usize, y: usize, z: usize) -> Voxel {
        let chunk_index = self.index_1d(
            x / self.chunk_size.0,
            y / self.chunk_size.1,
            z / self.chunk_size.2,
        );
        // Get the chunk indexes
        let xv = x % self.chunk_size.0;
        let yv = y % self.chunk_size.1;
        let zv = z % self.chunk_size.2;

        self.chunks[chunk_index].voxel(xv, yv, zv)
    }

    pub fn raw_voxel(&self, x: usize, y: usize, z: usize) -> u8 {
        let chunk_index = self.index_1d(
            x / self.chunk_size.0,
            y / self.chunk_size.1,
            z / self.chunk_size.2,
        );
        // Get the chunk indexes
        let xv = x % self.chunk_size.0;
        let yv = y % self.chunk_size.1;
        let zv = z % self.chunk_size.2;

        self.chunks[chunk_index].raw_voxel(xv, yv, zv)
    }

    pub fn calculate_distance_fields(&mut self) {}

    fn get_dist(&self, x: usize, y: usize, z: usize) -> u8 {
        1
    }

    fn set_distance_field(&mut self, x: usize, y: usize, z: usize, dist: u8) {
        let chunk_index = self.index_1d(x / self.x_depth, y / self.y_depth, z / self.z_depth);
        // Get the chunk indexes
        let xv = x % self.chunk_size.0;
        let yv = y % self.chunk_size.1;
        let zv = z % self.chunk_size.2;

        self.chunks[chunk_index].set_distance_field(xv, yv, zv, dist);
    }

    pub fn set_voxel(&mut self, x: usize, y: usize, z: usize, voxel: Voxel) {
        // Get the chunk indexes
        let chunk_index = self.index_1d(
            x / self.chunk_size.0,
            y / self.chunk_size.1,
            z / self.chunk_size.2,
        );
        // Set the single voxel
        {
            // Get the vert indexes
            let xv = x % self.chunk_size.0;
            let yv = y % self.chunk_size.1;
            let zv = z % self.chunk_size.2;

            self.chunks[chunk_index].set_voxel(xv, yv, zv, voxel);
        }

        // Set all voxels to 0 in this level field
        let level = 1;
        for z in self.safe_min(z, level)..self.safe_max(z, level, Axis::Z) {
            for y in self.safe_min(y, level)..self.safe_max(y, level, Axis::Y) {
                for x in self.safe_min(x, level)..self.safe_max(x, level, Axis::X) {
                    self.set_distance_field(x, y, z, 1);
                }
            }
        }

        // For all empty voxels within 1 of the voxel, set distance to 1
    }

    // Level = steps to take
    fn safe_min(&self, i: usize, level: usize) -> usize {
        if i == 0 {
            return 0;
        }

        return match i.checked_sub(level) {
            Some(i) => i,
            None => 0,
        };
    }
    fn safe_max(&self, i: usize, level: usize, axis: Axis) -> usize {
        let i = i + level;

        match axis {
            Axis::X => {
                let max = self.x_depth * self.chunk_size.0;
                if max < i {
                    return max;
                }

                return i;
            }
            Axis::Y => {
                let max = self.y_depth * self.chunk_size.1;
                if max < i {
                    return max;
                }

                return i;
            }
            Axis::Z => {
                let max = self.z_depth * self.chunk_size.2;
                if max < i {
                    return max;
                }

                return i;
            }
        }
    }
}

enum Axis {
    X,
    Y,
    Z,
}

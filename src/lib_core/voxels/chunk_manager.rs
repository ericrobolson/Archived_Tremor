use crate::lib_core::{math::index_1d, math::index_3d, time::GameFrame};

use super::{Chunk, Palette, PaletteIndex, Voxel};

pub struct ChunkManager {
    x_depth: usize,
    y_depth: usize,
    z_depth: usize,
    last_update: GameFrame,
    current_frame: GameFrame,
    pub chunks: Vec<Chunk>,
}

impl ChunkManager {
    pub fn new(x_depth: usize, y_depth: usize, z_depth: usize) -> Self {
        let capacity = x_depth * y_depth * z_depth;
        let mut chunks = Vec::with_capacity(capacity);

        let chunk_size = 16;

        for _ in 0..capacity {
            chunks.push(Chunk::new(chunk_size, chunk_size, chunk_size));
        }

        Self {
            x_depth,
            y_depth,
            z_depth,
            last_update: 0,
            current_frame: 0,
            chunks,
        }
    }

    pub fn last_update(&self) -> GameFrame {
        self.last_update
    }

    pub fn update(&mut self, frame: GameFrame) {
        self.current_frame = frame;
        for chunk in self.chunks.iter_mut() {
            chunk.update(frame);
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
}

use crate::lib_core::{math::index_1d, math::index_3d, time::GameFrame};

use super::{Chunk, ChunkMesh, Palette, PaletteIndex, Voxel};

pub struct ChunkManager {
    x_depth: usize,
    y_depth: usize,
    z_depth: usize,
    last_update: GameFrame,
    current_frame: GameFrame,
    chunks: Vec<Chunk>,
    pub meshes: Vec<ChunkMesh>,
}

impl ChunkManager {
    pub fn new(x_depth: usize, y_depth: usize, z_depth: usize) -> Self {
        let capacity = x_depth * y_depth * z_depth;
        let mut chunks = Vec::with_capacity(capacity);
        let mut meshes = Vec::with_capacity(capacity);

        let chunk_size = 16;

        for i in 0..capacity {
            chunks.push(Chunk::new(chunk_size, chunk_size, chunk_size));
            let mut mesh = ChunkMesh::from_chunk(&chunks[i]);
            // Scale the verts
            for j in 0..mesh.verts.len() / 3 {
                let (x, y, z) = (j * 3, j * 3 + 1, j * 3 + 2);

                let (chunk_x, chunk_y, chunk_z) = index_3d(i, x_depth, y_depth, z_depth);

                mesh.verts[x] += (chunk_x * chunk_size) as f32;
                mesh.verts[y] += (chunk_y * chunk_size) as f32;
                mesh.verts[z] += (chunk_z * chunk_size) as f32;
            }
            meshes.push(mesh);
        }

        Self {
            x_depth,
            y_depth,
            z_depth,
            last_update: 0,
            current_frame: 0,
            chunks,
            meshes,
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

    fn index_1d(&self, x: usize, y: usize, z: usize) -> usize {
        index_1d(x, y, z, self.x_depth, self.y_depth, self.z_depth)
    }

    fn index_3d(&self, i: usize) -> (usize, usize, usize) {
        index_3d(i, self.x_depth, self.y_depth, self.z_depth)
    }
}

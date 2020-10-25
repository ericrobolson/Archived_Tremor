use super::{Chunk, Voxel};
use crate::lib_core::time::GameFrame;

enum MeshingStrategy {
    Dumb,
}

pub struct ChunkMesh {
    pub verts: Vec<f32>,
    pub colors: Vec<f32>,
    last_update: GameFrame,
}

impl ChunkMesh {
    pub fn from_chunk(chunk: &Chunk) -> Self {
        let mut verts = vec![];
        let mut colors = vec![];

        let (x_size, y_size, z_size) = chunk.capacity();

        let meshing_strategy = MeshingStrategy::Dumb;

        match meshing_strategy {
            MeshingStrategy::Dumb => {
                for x in 0..x_size {
                    let xf32 = x as f32;
                    for y in 0..y_size {
                        let yf32 = y as f32;
                        for z in 0..z_size {
                            let zf32 = z as f32;

                            let voxel = chunk.voxel(x, y, z);
                            if voxel == Voxel::Empty {
                                continue;
                            }

                            let mut cube = ChunkMesh::cube_verts();
                            // adjust positions
                            let mut i = 0;
                            while i < cube.len() {
                                cube[i] += xf32;
                                cube[i + 1] += yf32;
                                cube[i + 2] += zf32;

                                i += 3;
                            }

                            colors
                                .append(&mut ChunkMesh::color_verts(cube.len(), voxel.to_color()));

                            verts.append(&mut cube);
                        }
                    }
                }
            }
        }

        Self {
            verts,
            colors,
            last_update: chunk.last_update(),
        }
    }

    fn cube_verts() -> Vec<f32> {
        let mut verts = vec![
            -1.0, -1.0, -1.0, // triangle 1 : begin
            -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, // triangle 1 : end
            1.0, 1.0, -1.0, // triangle 2 : begin
            -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, // triangle 2 : end
            1.0, -1.0, 1.0, -1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0, -1.0, -1.0,
            -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0,
            -1.0, -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0,
            1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, 1.0, 1.0,
            -1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, 1.0, -1.0, 1.0,
            -1.0, -1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0, 1.0, 1.0, 1.0, -1.0, 1.0,
        ];

        verts.iter().map(|v| v / 2.0).collect()
    }

    fn color_verts(len: usize, color: (f32, f32, f32)) -> Vec<f32> {
        let mut colors = Vec::with_capacity(len);

        for i in 0..len / 3 {
            colors.push(color.0);
            colors.push(color.1);
            colors.push(color.2);
        }

        colors
    }
}

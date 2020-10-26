use super::vertex::{Vertex, VoxelChunkVertex};
use crate::lib_core::{
    ecs::World,
    math::index_3d,
    time::GameFrame,
    voxels::{Chunk, ChunkManager, Voxel},
};

pub struct VoxelPass {
    meshes: Vec<Mesh>,
}

impl VoxelPass {
    pub fn new(device: &wgpu::Device) -> Self {
        //TODO: move to ecs
        let size = 4;
        let chunk_manager = ChunkManager::new(size, size, size);

        let mut meshes = Vec::with_capacity(chunk_manager.len());
        for i in 0..chunk_manager.chunks.len() {
            let mesh = Mesh::new(i, &chunk_manager, device);
            meshes.push(mesh);
        }

        Self { meshes }
    }

    pub fn update(&mut self, world: &World) {
        // TODO: update each chunk if changed
    }

    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        // Draw each chunk.
        // TODO: frustrum culling
        for mesh in &self.meshes {
            mesh.draw(render_pass);
        }
    }
}

enum MeshingStrategy {
    Dumb,
}

struct Mesh {
    chunk_index: usize,
    last_updated: GameFrame,
    vert_len: usize,
    buffer: wgpu::Buffer,
}

impl Mesh {
    fn new(chunk_index: usize, chunk_manager: &ChunkManager, device: &wgpu::Device) -> Self {
        let (cube_verts, color_verts) = Self::from_chunk(chunk_index, chunk_manager);

        let verts: Vec<VoxelChunkVertex> = VoxelChunkVertex::from_verts(cube_verts, color_verts);

        use wgpu::util::DeviceExt;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Voxel Verts"),
            contents: bytemuck::cast_slice(&verts),
            usage: wgpu::BufferUsage::VERTEX,
        });

        Self {
            chunk_index,
            vert_len: verts.len(),
            buffer,
            last_updated: 0,
        }
    }

    fn update(&mut self, chunk: &Chunk) {
        //TODO: pass in chunk manager and use that.
        if self.last_updated < chunk.last_update() {
            // TODO: remesh
        }
    }

    pub fn from_chunk(chunk_index: usize, chunk_manager: &ChunkManager) -> (Vec<f32>, Vec<f32>) {
        let mut verts = vec![];
        let mut colors = vec![];

        let chunk = &chunk_manager.chunks[chunk_index];

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

                            let mut cube = Self::cube_verts();
                            // adjust positions
                            let mut i = 0;
                            while i < cube.len() {
                                cube[i] += xf32;
                                cube[i + 1] += yf32;
                                cube[i + 2] += zf32;

                                i += 3;
                            }

                            colors.append(&mut Self::color_verts(cube.len(), voxel.to_color()));

                            verts.append(&mut cube);
                        }
                    }
                }
            }
        }

        let (chunk_x_size, chunk_y_size, chunk_z_size) = chunk.capacity();
        let (chunks_x_depth, chunks_y_depth, chunks_z_depth) = chunk_manager.capacity();

        // Iterate over each vertex (3 floats), adjusting its position
        for j in 0..verts.len() / 3 {
            let (x, y, z) = (j * 3, j * 3 + 1, j * 3 + 2);

            let (chunk_x, chunk_y, chunk_z) =
                index_3d(chunk_index, chunks_x_depth, chunks_y_depth, chunks_z_depth);

            verts[x] += (chunk_x * chunk_x_size) as f32;
            verts[y] += (chunk_y * chunk_y_size) as f32;
            verts[z] += (chunk_z * chunk_z_size) as f32;
        }

        (verts, colors)
    }

    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.buffer.slice(..));
        render_pass.draw(0..self.vert_len as u32, 0..1);
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

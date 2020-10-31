use wgpu::util::DeviceExt;

use super::vertex::Vertex;
use crate::lib_core::{
    ecs::World,
    math::index_3d,
    time::GameFrame,
    voxels::{Chunk, ChunkManager, Voxel},
};

pub mod texture_voxels;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct VoxelChunkVertex {
    position: [f32; 3],
    color: [f32; 3],
}

unsafe impl bytemuck::Pod for VoxelChunkVertex {}
unsafe impl bytemuck::Zeroable for VoxelChunkVertex {}

impl VoxelChunkVertex {
    pub fn from_verts(chunk_verts: Vec<f32>, color_verts: Vec<f32>) -> Vec<Self> {
        let mut verts = vec![];
        for i in 0..chunk_verts.len() / 3 {
            let j = i * 3;
            let (k, l, m) = (j, j + 1, j + 2);
            let pos: [f32; 3] = [chunk_verts[k], chunk_verts[l], chunk_verts[m]];
            let col: [f32; 3] = [color_verts[k], color_verts[l], color_verts[m]];

            verts.push(Self {
                position: pos,
                color: col,
            });
        }

        verts
    }
}

impl Vertex for VoxelChunkVertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<VoxelChunkVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float3,
                },
            ],
        }
    }
}

pub struct VoxelPass {
    meshes: Vec<Mesh>,
}

impl VoxelPass {
    pub fn new(world: &World, device: &wgpu::Device) -> Self {
        let chunk_manager = &world.world_voxels;

        let mut d = Vec::with_capacity(chunk_manager.len());
        for i in 0..chunk_manager.chunks.len() {
            d.push(i);
        }

        use rayon::prelude::*;
        let meshes = d
            .par_iter()
            .map(|i| Mesh::new(*i, &chunk_manager, device))
            .collect();

        Self { meshes }
    }

    pub fn update(&mut self, world: &World, device: &wgpu::Device, queue: &wgpu::Queue) {
        use rayon::prelude::*;

        self.meshes
            .par_iter_mut()
            .for_each(|m| m.update(&world.world_voxels, device, queue));
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
        let verts = Self::verts(chunk_index, chunk_manager);

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Voxel Verts"),
            contents: bytemuck::cast_slice(&verts),
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
        });

        Self {
            chunk_index,
            vert_len: verts.len(),
            buffer,
            last_updated: 0,
        }
    }

    fn update(&mut self, chunk_manager: &ChunkManager, device: &wgpu::Device, queue: &wgpu::Queue) {
        let chunk = &chunk_manager.chunks[self.chunk_index];
        return; //TODO: undo
                // Remesh if more recent
        if self.last_updated < chunk.last_update() {
            self.last_updated = chunk.last_update();
            let verts = Self::verts(self.chunk_index, chunk_manager);
            self.buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Voxel Verts"),
                contents: bytemuck::cast_slice(&verts),
                usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            });
        }
    }

    fn verts(chunk_index: usize, chunk_manager: &ChunkManager) -> Vec<VoxelChunkVertex> {
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

        VoxelChunkVertex::from_verts(verts, colors)
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

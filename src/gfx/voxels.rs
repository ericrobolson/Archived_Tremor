use super::vertex::{Vertex, VoxelChunkVertex};
use crate::lib_core::{
    ecs::World,
    time::GameFrame,
    voxels::{Chunk, ChunkManager, ChunkMesh},
};

pub struct VoxelPass {
    pub buffer: wgpu::Buffer,
    pub vert_len: usize,
    meshes: Vec<Mesh>,
}

impl VoxelPass {
    pub fn new(device: &wgpu::Device) -> Self {
        let size = 4;
        let chunk_manager = ChunkManager::new(size, size, size);
        let verts: Vec<VoxelChunkVertex> = chunk_manager
            .meshes
            .iter()
            .map(|c| VoxelChunkVertex::from_chunk(&c))
            .flatten()
            .collect();

        let meshes = Vec::with_capacity(chunk_manager.len());

        use wgpu::util::DeviceExt;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Voxel Verts"),
            contents: bytemuck::cast_slice(&verts),
            usage: wgpu::BufferUsage::VERTEX,
        });

        Self {
            buffer,
            vert_len: verts.len(),
            meshes,
        }
    }

    pub fn update(&mut self, world: &World) {
        // TODO: update each chunk
    }

    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        // TODO: change to use each individual chunk
        render_pass.set_vertex_buffer(0, self.buffer.slice(..));
        render_pass.draw(0..self.vert_len as u32, 0..1);
    }
}

struct Mesh {
    last_updated: GameFrame,
    verts: Vec<VoxelChunkVertex>,
}

impl Mesh {
    fn new() -> Self {
        unimplemented!();
    }

    fn update(&mut self, chunk: &Chunk) {
        if self.last_updated < chunk.last_update() {
            // TODO: remesh
        }
    }
}

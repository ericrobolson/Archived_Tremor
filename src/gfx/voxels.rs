use super::vertex::{Vertex, VoxelChunkVertex};
use crate::lib_core::{
    ecs::World,
    time::GameFrame,
    voxels::{Chunk, ChunkManager, ChunkMesh},
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
        for i in 0..chunk_manager.meshes.len() {
            let mesh = Mesh::new(&chunk_manager.meshes[i], device);
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

struct Mesh {
    last_updated: GameFrame,
    vert_len: usize,
    buffer: wgpu::Buffer,
}

impl Mesh {
    fn new(chunk: &ChunkMesh, device: &wgpu::Device) -> Self {
        let verts: Vec<VoxelChunkVertex> = VoxelChunkVertex::from_chunk(chunk);

        use wgpu::util::DeviceExt;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Voxel Verts"),
            contents: bytemuck::cast_slice(&verts),
            usage: wgpu::BufferUsage::VERTEX,
        });

        Self {
            vert_len: verts.len(),
            buffer,
            last_updated: 0,
        }
    }

    fn update(&mut self, chunk: &Chunk) {
        if self.last_updated < chunk.last_update() {
            // TODO: remesh
        }
    }

    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.buffer.slice(..));
        render_pass.draw(0..self.vert_len as u32, 0..1);
    }
}

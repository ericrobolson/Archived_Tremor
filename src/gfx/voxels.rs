use super::vertex::{Vertex, VoxelChunkVertex};
use crate::lib_core::voxels::{Chunk, ChunkMesh};

pub struct VoxelPass {
    pub buffer: wgpu::Buffer,
    pub vert_len: usize,
}

impl VoxelPass {
    pub fn new(device: &wgpu::Device) -> Self {
        let size = 16;
        let chunk = Chunk::new(size, size, size);
        let chunk_mesh = ChunkMesh::from_chunk(&chunk);
        let verts = VoxelChunkVertex::from_chunk(&chunk_mesh);

        use wgpu::util::DeviceExt;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Voxel Verts"),
            contents: bytemuck::cast_slice(&verts),
            usage: wgpu::BufferUsage::VERTEX,
        });

        Self {
            buffer,
            vert_len: verts.len(),
        }
    }
}

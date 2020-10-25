use super::vertex::{Vertex, VoxelChunkVertex};
use crate::lib_core::voxels::{ChunkManager, ChunkMesh};

pub struct VoxelPass {
    pub buffer: wgpu::Buffer,
    pub vert_len: usize,
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

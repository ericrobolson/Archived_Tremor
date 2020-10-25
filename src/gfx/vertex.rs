use crate::lib_core::voxels::ChunkMesh;

pub trait Vertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct VoxelChunkVertex {
    position: [f32; 3],
    color: [f32; 3],
}

unsafe impl bytemuck::Pod for VoxelChunkVertex {}
unsafe impl bytemuck::Zeroable for VoxelChunkVertex {}

impl VoxelChunkVertex {
    pub fn from_chunk(chunk: &ChunkMesh) -> Vec<Self> {
        let mut verts = vec![];

        println!("{:?}", chunk.verts.len());
        println!("{:?}", chunk.colors.len());

        for i in 0..chunk.verts.len() / 3 {
            let j = i * 3;
            let (k, l, m) = (j, j + 1, j + 2);
            let pos: [f32; 3] = [chunk.verts[k], chunk.verts[l], chunk.verts[m]];
            let col: [f32; 3] = [chunk.colors[k], chunk.colors[l], chunk.colors[m]];

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

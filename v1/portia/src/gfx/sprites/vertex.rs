use rendering_ir::wgpu_helpers::vertex::Vertex;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SpriteVert {
    pub position: [f32; 2],
    pub texture_coords: [f32; 2],
    pub color: [f32; 4],
}

unsafe impl bytemuck::Pod for SpriteVert {}
unsafe impl bytemuck::Zeroable for SpriteVert {}

impl Vertex for SpriteVert {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        use std::mem;
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<SpriteVert>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float2,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float2,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float4,
                },
            ],
        }
    }
}

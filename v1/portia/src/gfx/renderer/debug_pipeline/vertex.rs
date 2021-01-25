use rendering_ir::wgpu_helpers::vertex;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub color: [f32; 4],
}

impl vertex::Vertex for Vertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        use std::mem;
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                //          pub pos: [f32; 3],
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                //     pub normal: [f32; 3],
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float4,
                },
            ],
        }
    }
}

pub fn init_colored_quad(
    x: f32,
    y: f32,
    z: f32,
    width: f32,
    height: f32,
    color: [f32; 4],
    quad_offset: Option<vertex::Index>,
) -> (Vec<Vertex>, Vec<vertex::Index>) {
    let (vert, indexes) = vertex::indexed_quad(x, y, z, width, height, quad_offset);

    let verts = vert.iter().map(|v| Vertex { pos: *v, color });

    (verts.collect(), indexes)
}

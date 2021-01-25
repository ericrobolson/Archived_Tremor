use rendering_ir::wgpu_helpers::vertex;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub uv0: [f32; 2],
    pub uv1: [f32; 2],
    pub joint0: [f32; 4],
    pub weight0: [f32; 4],
}

type Vec2 = [f32; 2];
type Vec3 = [f32; 3];
type Vec4 = [f32; 4];

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
                    offset: mem::size_of::<Vec3>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float3,
                },
                //     pub uv0: [f32; 2],
                wgpu::VertexAttributeDescriptor {
                    offset: 2 * mem::size_of::<Vec3>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float2,
                },
                //     pub uv1: [f32; 2],
                wgpu::VertexAttributeDescriptor {
                    offset: 2 * mem::size_of::<Vec3>() as wgpu::BufferAddress
                        + 1 * mem::size_of::<Vec2>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float2,
                },
                //    pub joint0: [f32; 4],
                wgpu::VertexAttributeDescriptor {
                    offset: 2 * mem::size_of::<Vec3>() as wgpu::BufferAddress
                        + 2 * mem::size_of::<Vec2>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float4,
                },
                //pub weight0: [f32; 4],
                wgpu::VertexAttributeDescriptor {
                    offset: 2 * mem::size_of::<Vec3>() as wgpu::BufferAddress
                        + 2 * mem::size_of::<Vec2>() as wgpu::BufferAddress
                        + 1 * mem::size_of::<Vec4>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float4,
                },
            ],
        }
    }
}

pub struct Vertexes {
    buffer: wgpu::Buffer,
}

pub struct Indices {
    count: u32,
    buffer: wgpu::Buffer,
}

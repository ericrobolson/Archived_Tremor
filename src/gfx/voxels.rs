use cgmath::{Vector2, Vector3};
use wgpu::util::DeviceExt;
use wgpu::vertex_attr_array;

use super::model::Vertex;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Materials {
    Empty,
    Dirt,
    Metal,
    Stone,
    Wood,
    Water,
    Gas,
}

const CAPACITY: usize = 16;
const CAPACITY_CUBED: usize = CAPACITY * CAPACITY * CAPACITY;
pub struct VoxelChunk {
    voxels: Vec<Materials>,
    position: (f32, f32, f32),
}

impl VoxelChunk {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        let (x, y, z) = {
            // Hacky fix for chunk positions
            let cap = CAPACITY as f32;
            (x * cap, y * cap, z * cap)
        };
        let mut voxels = Vec::with_capacity(CAPACITY_CUBED);
        for i in 0..CAPACITY_CUBED {
            let mut mat = Materials::Empty;

            if i % 2 == 0 {
                mat = Materials::Stone;
            }

            voxels.push(mat);
        }

        if voxels.len() != CAPACITY_CUBED {
            panic!("VOXEL LEN NOT EQUAL TO CAPACITY!");
        }

        Self {
            voxels,
            position: (x, y, z),
        }
    }

    pub fn voxel(&self, x: usize, y: usize, z: usize) -> Materials {
        self.voxels[((x * CAPACITY) * CAPACITY) + (y * CAPACITY) + z]
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct VoxelVertex {
    position: Vector3<f32>,
}

unsafe impl bytemuck::Pod for VoxelVertex {}
unsafe impl bytemuck::Zeroable for VoxelVertex {}

impl Vertex for VoxelVertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<VoxelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[wgpu::VertexAttributeDescriptor {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float3,
            }],
        }
    }
}

pub struct VoxelMesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub vert_count: usize,
}

impl VoxelMesh {
    pub fn new(chunk: &VoxelChunk, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let mut vertices = vec![];

        for x in 0..CAPACITY {
            for y in 0..CAPACITY {
                for z in 0..CAPACITY {
                    let v = chunk.voxel(x, y, z);
                    if v != Materials::Empty {
                        // Dummy vert positions for a simple cube, not mapped to world space
                        let verts: Vec<f32> = vec![
                            -1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0, -1.0,
                            -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0, -1.0, -1.0,
                            1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, -1.0,
                            -1.0, -1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0,
                            -1.0, -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0,
                            1.0, -1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0,
                            -1.0, -1.0, 1.0, 1.0, 1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
                            -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, 1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0,
                            1.0, 1.0, 1.0, -1.0, 1.0, 1.0, 1.0, -1.0, 1.0,
                        ];

                        let mut positions = vec![];
                        for i in 0..(verts.len() / 3) {
                            let index = i * 3;
                            // Get every 3 verts, and make an array. Also divide by two, as it's pretty massive.
                            positions.push([
                                verts[index] / 2.0,
                                verts[index + 1] / 2.0,
                                verts[index + 2] / 2.0,
                            ]);
                        }
                        let (xpos, ypos, zpos) = chunk.position;
                        let mut verts: Vec<VoxelVertex> = positions
                            .iter()
                            .map(|pos| {
                                let mut pos = *pos;
                                pos[0] += (x as f32) + xpos;
                                pos[1] += (y as f32) + ypos;
                                pos[2] += (z as f32) + zpos;
                                pos
                            })
                            .map(|pos| VoxelVertex {
                                position: pos.into(),
                            })
                            .collect();

                        vertices.append(&mut verts);
                    }
                }
            }
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Voxel Vertex Buffer")),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsage::VERTEX,
        });

        Self {
            vertex_buffer,
            vert_count: vertices.len(),
            name: "TestVoxelMesh".into(),
        }
    }
}

pub trait DrawVoxels<'a, 'b>
where
    'b: 'a,
{
    fn draw_chunk(&mut self, chunk: &'b VoxelMesh, uniforms: &'b wgpu::BindGroup);
}

impl<'a, 'b> DrawVoxels<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_chunk(&mut self, chunk: &'b VoxelMesh, uniforms: &'b wgpu::BindGroup) {
        self.set_vertex_buffer(0, chunk.vertex_buffer.slice(..));
        self.set_bind_group(0, &uniforms, &[]);
        self.draw(0..chunk.vert_count as u32, 0..1);
    }
}

use rayon::prelude::*;
use wgpu::util::DeviceExt;

use super::{model_transform::ModelTransform, poly_renderer::BindGroups, vertex::Vertex};

use crate::lib_core::{
    ecs::{Entity, Mask, MaskType, World},
    spatial,
    time::GameFrame,
    voxels::{Chunk, Voxel},
};

pub mod palette;
pub mod texture_voxels;

type PaletteIndexType = u32;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct VoxelChunkVertex {
    position: [f32; 3],
    palette_index: PaletteIndexType,
}

unsafe impl bytemuck::Pod for VoxelChunkVertex {}
unsafe impl bytemuck::Zeroable for VoxelChunkVertex {}

impl VoxelChunkVertex {
    pub fn from_verts(chunk_verts: Vec<f32>, palette_indices: Vec<u8>) -> Vec<Self> {
        let mut verts = vec![];
        for i in 0..chunk_verts.len() / 3 {
            let j = i * 3;
            let (k, l, m) = (j, j + 1, j + 2);
            let pos: [f32; 3] = [chunk_verts[k], chunk_verts[l], chunk_verts[m]];

            let palette_index = palette_indices[i];

            verts.push(Self {
                position: pos,
                palette_index: palette_index as PaletteIndexType,
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
                    format: wgpu::VertexFormat::Uint,
                },
            ],
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum DoubleBuffer {
    Draw0Update1,
    Draw1Update0,
}

impl DoubleBuffer {
    pub fn swap(&self) -> DoubleBuffer {
        match self {
            DoubleBuffer::Draw0Update1 => DoubleBuffer::Draw1Update0,
            DoubleBuffer::Draw1Update0 => DoubleBuffer::Draw0Update1,
        }
    }
}

pub struct VoxelPass {
    meshes: Vec<(Mesh, Mesh)>,
    double_buffer: DoubleBuffer,
}

const VOXEL_PASS_MASK: MaskType = Mask::TRANSFORM & Mask::VOXEL_CHUNK;
fn active_entity(entity: Entity, world: &World) -> bool {
    return world.masks[entity] & VOXEL_PASS_MASK == VOXEL_PASS_MASK;
}

impl VoxelPass {
    pub fn new(
        world: &World,
        bind_groups: &BindGroups,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        // Iterate over all entities, regardless of whether it's active or not. Creates everything at once for minimal memory allocation.
        let entities = (0..world.max_entities()).collect::<Vec<usize>>();
        let meshes: Vec<(Mesh, Mesh)> = entities
            .par_iter()
            .map(|entity| {
                let entity = *entity;

                // Convert entity into mesh. Right now just initializing everything at once instead of doing it piecemeal
                // Create 2 meshes per entity. This is used for 'double buffering'. E.g. update mesh 1 and draw mesh 0, then next frame update mesh 0 and draw 1, repeat ad nauseam.
                let chunk = &world.voxel_chunks[entity];
                let transform = &world.transforms[entity];
                let is_active = active_entity(entity, world);
                let mesh0 = Mesh::new(
                    entity,
                    is_active,
                    chunk,
                    transform,
                    bind_groups,
                    device,
                    queue,
                );
                let mesh1 = Mesh::new(
                    entity,
                    is_active,
                    chunk,
                    transform,
                    bind_groups,
                    device,
                    queue,
                );

                (mesh0, mesh1)
            })
            .collect();
        Self {
            meshes,
            double_buffer: DoubleBuffer::Draw0Update1,
        }
    }

    pub fn update(&mut self, world: &World, device: &wgpu::Device, queue: &wgpu::Queue) {
        // Change which buffer we're updating and drawing
        self.double_buffer = self.double_buffer.swap();

        let double_buffer = self.double_buffer;

        // Update all meshes
        self.meshes.par_iter_mut().for_each(|(m0, m1)| {
            let entity = m0.entity;

            let chunk = &world.voxel_chunks[entity];
            let transform = &world.transforms[entity];
            let is_active = active_entity(entity, world);

            if double_buffer == DoubleBuffer::Draw1Update0 {
                // Update 0
                m0.update(is_active, chunk, transform, device, queue);
            } else {
                // Update 1
                m1.update(is_active, chunk, transform, device, queue);
            }
        });
    }

    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        // Draw each chunk.
        // TODO: frustrum culling
        for (m0, m1) in &self.meshes {
            if self.double_buffer == DoubleBuffer::Draw0Update1 {
                // Draw 0
                m0.draw(render_pass);
            } else {
                // Draw 1
                m1.draw(render_pass);
            }
        }
    }
}

struct MeshBufferVerts {
    max_capacity: (usize, usize, usize),
    vert_len: usize,
    buffer: wgpu::Buffer,
}

fn create_mesh_buffer_verts(
    chunk: &Chunk,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> MeshBufferVerts {
    // Calculate total length of buffer e.g. a full chunk of different voxels. This way a new buffer only has to be created when the voxel capacity is changed.

    let verts = Mesh::verts(chunk);
    let vert_len = verts.len();

    let single_cube_verts = Mesh::cube_verts().len();
    let single_cube_color_verts = (single_cube_verts / 3) * std::mem::size_of::<PaletteIndexType>(); // One PaletteIndexType per 3 verts

    let max_voxels = {
        let (x, y, z) = chunk.capacity();
        x * y * z
    };

    let max_buf_size =
        (single_cube_verts + single_cube_color_verts) * max_voxels * std::mem::size_of::<f32>();

    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        mapped_at_creation: false,
        size: max_buf_size as u64,
        usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
    });

    if vert_len > 0 {
        queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&verts));
    }

    MeshBufferVerts {
        buffer,
        vert_len,
        max_capacity: chunk.capacity(),
    }
}

enum MeshingStrategy {
    Dumb,
    RunLength,
    Greedy,
}

struct Mesh {
    entity: usize,
    last_updated: GameFrame,
    vert_len: usize,
    mesh_buffer: wgpu::Buffer,
    max_voxel_capacity: (usize, usize, usize),
    transform_buffer: wgpu::Buffer,
    transform_bind_group: wgpu::BindGroup,
    last_transform: spatial::Transformation,
    active: bool,
}

impl Mesh {
    fn new(
        entity: usize,
        active: bool,
        chunk: &Chunk,
        transform: &spatial::Transformation,
        bind_groups: &BindGroups,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let (mesh_buffer, vert_len, max_voxel_capacity) = {
            let data = create_mesh_buffer_verts(chunk, device, queue);
            (data.buffer, data.vert_len, data.max_capacity)
        };

        let (transform_bind_group, transform_buffer) = {
            let transform = ModelTransform::new(*transform);

            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Transform Buffer"),
                contents: bytemuck::cast_slice(&[transform]),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            });

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_groups.model_transform_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(buffer.slice(..)),
                }],
                label: Some("transform_bind_group"),
            });

            (bind_group, buffer)
        };

        Self {
            active,
            entity,
            vert_len,
            mesh_buffer,
            max_voxel_capacity,
            transform_bind_group,
            transform_buffer,
            last_transform: *transform,
            last_updated: 0,
        }
    }

    fn update(
        &mut self,
        active: bool,
        chunk: &Chunk,
        transform: &spatial::Transformation,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        self.active = active;

        if !self.active {
            return;
        }

        // If chunk capacity was changed, redo the buffer
        if chunk.capacity() != self.max_voxel_capacity {
            let (mesh_buffer, vert_len, max_voxel_capacity) = {
                let data = create_mesh_buffer_verts(chunk, device, queue);
                (data.buffer, data.vert_len, data.max_capacity)
            };

            self.vert_len = vert_len;
            self.mesh_buffer = mesh_buffer;
            self.max_voxel_capacity = max_voxel_capacity;
            self.last_updated = chunk.last_update();
        } else
        // Remesh if more recent
        if self.last_updated < chunk.last_update() {
            self.last_updated = chunk.last_update();
            let verts = Self::verts(chunk);
            self.vert_len = verts.len();

            if verts.len() > 0 {
                queue.write_buffer(&self.mesh_buffer, 0, bytemuck::cast_slice(&verts));
            }
        }

        // Update transform if different
        if *transform != self.last_transform {
            let transform = ModelTransform::new(*transform);
            queue.write_buffer(
                &self.transform_buffer,
                0,
                bytemuck::cast_slice(&[transform]),
            );
        }
    }

    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.vert_len > 0 && self.active {
            render_pass.set_bind_group(
                BindGroups::MODEL_TRANSFORM,
                &self.transform_bind_group,
                &[],
            );
            render_pass.set_vertex_buffer(0, self.mesh_buffer.slice(..));
            render_pass.draw(0..self.vert_len as u32, 0..1);
        }
    }

    fn cube_verts() -> Vec<f32> {
        vec![
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
        ]
        .iter()
        .map(|v| v / 2.0) // Need to divide it in half otherwise it's too large.
        .collect()
    }

    fn verts(chunk: &Chunk) -> Vec<VoxelChunkVertex> {
        let mut verts = vec![];
        let mut palette_colors = vec![];

        let (x_size, y_size, z_size) = chunk.capacity();

        let meshing_strategy = MeshingStrategy::RunLength;

        match meshing_strategy {
            MeshingStrategy::Dumb => {
                for z in 0..z_size {
                    let zf32 = z as f32;
                    for y in 0..y_size {
                        let yf32 = y as f32;
                        // TODO: run length encoding here.
                        for x in 0..x_size {
                            let xf32 = x as f32;

                            let voxel = chunk.voxel(x, y, z);
                            if voxel == Voxel::Empty || chunk.occluded(x, y, z) {
                                continue;
                            }

                            let mut cube = Self::cube_verts();
                            let mut i = 0;
                            while i < cube.len() {
                                // adjust positions
                                cube[i] += xf32;
                                cube[i + 1] += yf32;
                                cube[i + 2] += zf32;

                                // Add palette color for this vert
                                palette_colors.push(voxel.palette_index());

                                i += 3;
                            }

                            verts.append(&mut cube);
                        }
                    }
                }
            }
            MeshingStrategy::RunLength => {
                for z in 0..z_size {
                    let zf32 = z as f32;
                    for y in 0..y_size {
                        let yf32 = y as f32;
                        // TODO: run length encoding here.
                        for x in 0..x_size {
                            let xf32 = x as f32;

                            let voxel = chunk.voxel(x, y, z);
                            if voxel == Voxel::Empty || chunk.occluded(x, y, z) {
                                continue;
                            }

                            let mut cube = Self::cube_verts();
                            let mut i = 0;
                            while i < cube.len() {
                                // adjust positions
                                cube[i] += xf32;
                                cube[i + 1] += yf32;
                                cube[i + 2] += zf32;

                                // Add palette color for this vert
                                palette_colors.push(voxel.palette_index());

                                i += 3;
                            }

                            verts.append(&mut cube);
                        }
                    }
                }
            }
            MeshingStrategy::Greedy => {
                unimplemented!();
            }
        }

        VoxelChunkVertex::from_verts(verts, palette_colors)
    }
}

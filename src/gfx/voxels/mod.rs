use rayon::prelude::*;
use wgpu::util::DeviceExt;

use super::{
    model_transform::ModelTransform, poly_renderer::BindGroups, texture::Texture, vertex::Vertex,
};

use crate::lib_core::{
    ecs::World,
    math::{index_3d, Vec3},
    spatial,
    time::{Clock, Duration, GameFrame, Timer},
    voxels::{Chunk, ChunkManager, Color, Palette, Voxel},
};

pub mod palette;
pub mod texture_voxels;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct VoxelChunkVertex {
    position: [f32; 3],
    palette_index: u8,
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
                palette_index,
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

pub struct VoxelPass {
    meshes: Vec<Mesh>,
    last_updated_mesh: usize,
}

impl VoxelPass {
    pub fn new(
        world: &World,
        bind_groups: &BindGroups,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let chunk_manager = &world.world_voxels;

        let mut d = Vec::with_capacity(chunk_manager.len());
        for i in 0..chunk_manager.chunks.len() {
            d.push(i);
        }

        // TODO: remove old meshing. Instead if wanting to use chunk managers, create a voxel trait and inherit chunks and chunk managers?

        let meshes = d
            .par_iter()
            .map(|i| Mesh::new(*i, &chunk_manager, bind_groups, device, queue))
            .collect();

        // Iterate over all entities, regardless of whether it's active or not. Creates everything at once for minimal memory allocation.
        let entities = (0..world.max_entities()).collect::<Vec<usize>>();
        let d: Vec<usize> = entities
            .par_iter()
            .map(|entity| {
                let entity = *entity;

                // Convert entity into mesh. Right now just initializing everything at once instead of doing it piecemeal
                // TODO: Create meshes from entities.
                let mesh = ();

                entity
            })
            .collect();
        Self {
            meshes,
            last_updated_mesh: 0,
        }
    }

    pub fn update(&mut self, world: &World, device: &wgpu::Device, queue: &wgpu::Queue) {
        let stopwatch = Clock::new();
        const nanoseconds_in_second: u32 = 1000000000;
        const max_run_time: u32 = nanoseconds_in_second / 120;
        let target_duration = Duration::new(0, max_run_time);

        use rayon::prelude::*;
        self.meshes.par_iter_mut().for_each(|m| {
            if target_duration < stopwatch.elapsed() {
                return;
            }
            m.update(&world.world_voxels, device, queue);
        });
    }

    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        // Draw each chunk.
        // TODO: frustrum culling

        for mesh in &self.meshes {
            mesh.draw(render_pass);
        }
    }
}

enum MeshingStrategy {
    Dumb,
    RunLength,
    Greedy,
}

struct Mesh {
    chunk_index: usize,
    last_updated: GameFrame,
    vert_len: usize,
    mesh_buffer: wgpu::Buffer,
}

impl Mesh {
    fn new(
        chunk_index: usize,
        chunk_manager: &ChunkManager,
        bind_groups: &BindGroups,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let verts = Self::verts(chunk_index, chunk_manager);
        let vert_len = verts.len();

        // create and init buffer at max length.
        let mesh_buffer = {
            /*
            1) Calculate total length of buffer e.g. a full chunk of different voxels
            2) create buffer
            3) if data exists, update buffer
            4) write only active data when drawing
            5) update buffer instead of creating a new one
            */

            let singe_cube_verts = Self::cube_verts().len();
            let single_cube_colors = Self::color_verts(singe_cube_verts, (0.0, 0.0, 0.0)).len(); //TODO: think this is slightly wrong. May only need one per vert?

            let (x, y, z) = chunk_manager.chunk_size;
            let max_voxels = x * y * z;

            let max_buf_size =
                (singe_cube_verts + single_cube_colors) * max_voxels * std::mem::size_of::<f32>();

            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                mapped_at_creation: false,
                size: max_buf_size as u64,
                usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            });

            if vert_len > 0 {
                queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&verts));
            }

            buffer
        };

        let transform_buffer = {
            //TODO: replace with actual data

            let transform = ModelTransform::new(spatial::Transformation::default());

            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Transform Buffer"),
                contents: bytemuck::cast_slice(&[transform]),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            });

            let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_groups.model_transform_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(buffer.slice(..)),
                }],
                label: Some("uniform_bind_group"),
            });
        };

        Self {
            chunk_index,
            vert_len,
            mesh_buffer,
            last_updated: 0,
        }
    }

    fn update(&mut self, chunk_manager: &ChunkManager, device: &wgpu::Device, queue: &wgpu::Queue) {
        let chunk = &chunk_manager.chunks[self.chunk_index];

        //TODO: when using ECS, what happens if the max chunk size increases?

        // Remesh if more recent
        if self.last_updated < chunk.last_update() {
            self.last_updated = chunk.last_update();
            let verts = Self::verts(self.chunk_index, chunk_manager);
            self.vert_len = verts.len();

            if verts.len() > 0 {
                queue.write_buffer(&self.mesh_buffer, 0, bytemuck::cast_slice(&verts));
            }
        }
    }

    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.vert_len > 0 {
            render_pass.set_vertex_buffer(0, self.mesh_buffer.slice(..));
            render_pass.draw(0..self.vert_len as u32, 0..1);
        }
    }

    fn cube_verts() -> Vec<f32> {
        let mut verts = vec![
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
        ];

        verts.iter().map(|v| v / 2.0).collect()
    }

    fn color_verts(len: usize, color: (f32, f32, f32)) -> Vec<f32> {
        let mut colors = Vec::with_capacity(len);

        for i in 0..len / 3 {
            colors.push(color.0);
            colors.push(color.1);
            colors.push(color.2);
        }

        colors
    }

    fn verts(chunk_index: usize, chunk_manager: &ChunkManager) -> Vec<VoxelChunkVertex> {
        let mut verts = vec![];
        let mut palette_colors = vec![];

        let chunk = &chunk_manager.chunks[chunk_index];

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

        let (chunk_x_size, chunk_y_size, chunk_z_size) = chunk.capacity();
        let (chunks_x_depth, chunks_y_depth, chunks_z_depth) = chunk_manager.capacity();

        // Iterate over each vertex (3 floats), adjusting its position
        // TODO: this should be a uniform
        for j in 0..verts.len() / 3 {
            let (x, y, z) = (j * 3, j * 3 + 1, j * 3 + 2);

            let (chunk_x, chunk_y, chunk_z) =
                index_3d(chunk_index, chunks_x_depth, chunks_y_depth, chunks_z_depth);

            verts[x] += (chunk_x * chunk_x_size) as f32;
            verts[y] += (chunk_y * chunk_y_size) as f32;
            verts[z] += (chunk_z * chunk_z_size) as f32;
        }

        VoxelChunkVertex::from_verts(verts, palette_colors)
    }
}

/* Old mesh
struct Mesh {
    chunk_index: usize,
    last_updated: GameFrame,
    vert_len: usize,
    buffer: wgpu::Buffer,
}

impl Mesh {
    fn new(
        chunk_index: usize,
        chunk_manager: &ChunkManager,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let verts = Self::verts(chunk_index, chunk_manager);
        let vert_len = verts.len();

        // create and init buffer at max length.
        let buffer = {
            /*
            1) Calculate total length of buffer e.g. a full chunk of different voxels
            2) create buffer
            3) if data exists, update buffer
            4) write only active data when drawing
            5) update buffer instead of creating a new one
            */

            let singe_cube_verts = Self::cube_verts().len();
            let single_cube_colors = Self::color_verts(singe_cube_verts, (0.0, 0.0, 0.0)).len(); //TODO: think this is slightly wrong. May only need one per vert?

            let (x, y, z) = chunk_manager.chunk_size;
            let max_voxels = x * y * z;

            let max_buf_size =
                (singe_cube_verts + single_cube_colors) * max_voxels * std::mem::size_of::<f32>();

            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                mapped_at_creation: false,
                size: max_buf_size as u64,
                usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            });

            if vert_len > 0 {
                queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&verts));
            }

            buffer
        };

        Self {
            chunk_index,
            vert_len,
            buffer,
            last_updated: 0,
        }
    }

    fn update(&mut self, chunk_manager: &ChunkManager, device: &wgpu::Device, queue: &wgpu::Queue) {
        let chunk = &chunk_manager.chunks[self.chunk_index];

        //TODO: when using ECS, what happens if the max chunk size increases?

        // Remesh if more recent
        if self.last_updated < chunk.last_update() {
            self.last_updated = chunk.last_update();
            let verts = Self::verts(self.chunk_index, chunk_manager);
            self.vert_len = verts.len();

            if verts.len() > 0 {
                queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&verts));
            }
        }
    }

    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.vert_len > 0 {
            render_pass.set_vertex_buffer(0, self.buffer.slice(..));
            render_pass.draw(0..self.vert_len as u32, 0..1);
        }
    }

    fn cube_verts() -> Vec<f32> {
        let mut verts = vec![
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
        ];

        verts.iter().map(|v| v / 2.0).collect()
    }

    fn color_verts(len: usize, color: (f32, f32, f32)) -> Vec<f32> {
        let mut colors = Vec::with_capacity(len);

        for i in 0..len / 3 {
            colors.push(color.0);
            colors.push(color.1);
            colors.push(color.2);
        }

        colors
    }

    fn verts(chunk_index: usize, chunk_manager: &ChunkManager) -> Vec<VoxelChunkVertex> {
        let mut verts = vec![];
        let mut palette_colors = vec![];

        let chunk = &chunk_manager.chunks[chunk_index];

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

        let (chunk_x_size, chunk_y_size, chunk_z_size) = chunk.capacity();
        let (chunks_x_depth, chunks_y_depth, chunks_z_depth) = chunk_manager.capacity();

        // Iterate over each vertex (3 floats), adjusting its position
        // TODO: this should be a uniform
        for j in 0..verts.len() / 3 {
            let (x, y, z) = (j * 3, j * 3 + 1, j * 3 + 2);

            let (chunk_x, chunk_y, chunk_z) =
                index_3d(chunk_index, chunks_x_depth, chunks_y_depth, chunks_z_depth);

            verts[x] += (chunk_x * chunk_x_size) as f32;
            verts[y] += (chunk_y * chunk_y_size) as f32;
            verts[z] += (chunk_z * chunk_z_size) as f32;
        }

        VoxelChunkVertex::from_verts(verts, palette_colors)
    }
}

*/

use wgpu::util::DeviceExt;

use crate::lib_core::{
    ecs::World,
    math::index_3d,
    time::{GameFrame, Timer},
    voxels::{Chunk, ChunkManager, Voxel},
};

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct VoxelSpaceUniforms {
    // How big each voxel is in the world
    pub voxel_resolution: f32,
    // The number of voxels in the world
    pub voxel_world_size: [f32; 3],
}

unsafe impl bytemuck::Pod for VoxelSpaceUniforms {}
unsafe impl bytemuck::Zeroable for VoxelSpaceUniforms {}

impl VoxelSpaceUniforms {
    pub fn from_chunk_manager(chunk_manager: &ChunkManager) -> Self {
        let (voxels_width, voxels_height, voxels_depth) = voxels_count(chunk_manager);
        let voxels_width = voxels_width as f32;
        let voxels_height = voxels_height as f32;
        let voxels_depth = voxels_depth as f32;

        let voxel_resolution = chunk_manager.voxel_resolution.into();
        Self {
            voxel_resolution,
            voxel_world_size: [voxels_width, voxels_height, voxels_depth],
        }
    }
}

pub struct VoxelPass {
    pub volume_tex: Voxel3dTexture,
    pub voxel_uniforms: VoxelSpaceUniforms,
    last_update: GameFrame,
    timer: Timer,
}

impl VoxelPass {
    pub fn new(world: &World, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        // TODO: what to do if texture size is smaller than number of voxels?
        // TODO: What if maps change?
        // TODO: What if user screen size changes or they need more performance?
        let chunk_manager = &world.world_voxels;

        let volume_tex =
            Voxel3dTexture::new(&chunk_manager, device, queue, "VoxelTexture3d").unwrap();

        let voxel_uniforms = VoxelSpaceUniforms::from_chunk_manager(chunk_manager);

        let last_update = chunk_manager.chunks[0].last_update();
        Self {
            volume_tex,
            voxel_uniforms,
            last_update,
            timer: Timer::new(1),
        }
    }

    pub fn update(&mut self, world: &World, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.timer.can_run() {
            let mut should_update = false;
            for chunk in &world.world_voxels.chunks {
                if self.last_update <= chunk.last_update() {
                    should_update = true;
                    self.last_update = chunk.last_update();
                }
            }

            if should_update {
                self.volume_tex.update(&world.world_voxels, queue);
            }
        }
    }

    pub fn draw<'a>(&'a self, bind_group_index: u32, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_bind_group(bind_group_index, &self.volume_tex.bind_group, &[]);
    }
}

pub struct Voxel3dTexture {
    size: (usize, usize, usize),
    bytes_per_element: usize,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    format: wgpu::TextureFormat,
}

fn size_to_extent(size: (usize, usize, usize)) -> wgpu::Extent3d {
    let (height, width, depth) = size;
    wgpu::Extent3d {
        height: height as u32,
        width: width as u32,
        depth: depth as u32,
    }
}

fn voxels_count(chunk_manager: &ChunkManager) -> (usize, usize, usize) {
    let (width, height, depth) = chunk_manager.capacity();
    let (chunk_width, chunk_height, chunk_depth) = chunk_manager.chunk_size;

    (
        chunk_width * width,
        chunk_height * height,
        chunk_depth * depth,
    )
}

fn voxels(chunk_manager: &ChunkManager) -> Vec<u8> {
    let capacity = chunk_manager.capacity();
    let chunk_size = chunk_manager.chunk_size;

    // Voxels stored as u8's naturally. LSB = 0 is empty, other bits are distances. LSB = 1 is active, other bits are material index
    // Always ensure that it's read this way so it matches the texture layout.
    let mut data = vec![]; // TODO: Reserve?

    for z in 0..capacity.2 * chunk_size.2 {
        for y in 0..capacity.1 * chunk_size.1 {
            for x in 0..capacity.0 * chunk_size.0 {
                data.push(chunk_manager.raw_voxel(x, y, z));
            }
        }
    }
    data
}

impl Voxel3dTexture {
    pub fn new(
        chunk_manager: &ChunkManager,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: &str,
    ) -> Result<Self, String> {
        // Use signed 8bit ints to represent voxels. Enables SDFs and materials.
        let texture_format = wgpu::TextureFormat::R8Uint;
        let bytes_per_element = std::mem::size_of::<u8>();
        let bytes_per_element = 1;

        // Create 3d texture from
        let data: Vec<u8> = voxels(chunk_manager);

        return Self::from_bytes(
            voxels_count(chunk_manager),
            texture_format,
            device,
            queue,
            bytes_per_element,
            &data,
            label,
        );
    }
    pub fn from_bytes(
        size: (usize, usize, usize),
        format: wgpu::TextureFormat,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes_per_item: usize,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self, String> {
        let extent = size_to_extent(size);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: format,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            label: Some(label),
        });
        queue.write_texture(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &bytes,
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: (bytes_per_item as u32) * extent.width,
                rows_per_image: extent.depth,
            },
            extent,
        );
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            dimension: Some(wgpu::TextureViewDimension::D3),
            ..wgpu::TextureViewDescriptor::default()
        });

        let addr_mode = wgpu::AddressMode::ClampToEdge; // Prev clamp to edge

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: addr_mode,
            address_mode_v: addr_mode,
            address_mode_w: addr_mode,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::D3,
                            component_type: wgpu::TextureComponentType::Uint,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                        count: None,
                    },
                ],
                label: Some("voxel_texture_bind_group_layout"),
            });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("voxel_texture_bind_group"),
        });

        Ok(Self {
            format,
            size,
            texture,
            view: texture_view,
            bytes_per_element: bytes_per_item,
            sampler,
            texture_bind_group_layout,
            bind_group,
        })
    }

    pub fn update(&mut self, chunk_manager: &ChunkManager, queue: &wgpu::Queue) {
        let bytes: Vec<u8> = voxels(chunk_manager);

        let extent = size_to_extent(self.size);
        queue.write_texture(
            wgpu::TextureCopyView {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &bytes,
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: (self.bytes_per_element as u32) * extent.width,
                rows_per_image: extent.depth,
            },
            extent,
        );
    }
}

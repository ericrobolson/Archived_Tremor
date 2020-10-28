use wgpu::util::DeviceExt;

use crate::lib_core::{
    ecs::World,
    math::index_3d,
    time::GameFrame,
    voxels::{Chunk, ChunkManager, Voxel},
};

pub struct VoxelSpaceUniforms {
    // How big each voxel is in the world
    pub voxel_resolution: f32,
    // The number of voxels in the world
    pub voxel_world_size: [f32; 3],
    // The size of the world
    pub world_size: [f32; 3],
}

pub struct VoxelPass {}

impl VoxelPass {
    pub fn new(world: &World, device: &wgpu::Device) -> Self {
        let chunk_manager = &world.world_voxels;

        // Create 3d texture from voxels
        let (width, height, depth) = chunk_manager.capacity();

        // Raymarch the texture. Uses a simple

        Self {}
    }

    pub fn update(&mut self, world: &World, device: &wgpu::Device, queue: &wgpu::Queue) {}

    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {}
}

pub struct Texture3d {
    size: (usize, usize, usize),
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

impl Texture3d {
    pub fn new(
        size: (usize, usize, usize),
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: &str,
    ) -> Result<Self, String> {
        // Use signed 8bit ints to represent voxels. Enables SDFs and materials.
        let texture_format = wgpu::TextureFormat::R8Sint;
        let bytes_per_element = std::mem::size_of::<i8>();

        // TODO: convert voxels to i8's
        // LSB = 0 means it's not active. Can shift over to get the distance to the nearest voxel.
        // LSB = 1 means it's active. Uses an unsigned (meaning it's never negative) value after shifting 1 to read a texture to get the material values. RGBA for material texture.
        let data = vec![];

        return Self::from_bytes(
            size,
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

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
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
                            component_type: wgpu::TextureComponentType::Float,
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
                label: Some("texture_bind_group_layout"),
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
            label: Some("uniform_bind_group"),
        });

        Ok(Self {
            format,
            size,
            texture,
            view: texture_view,
            sampler,
            texture_bind_group_layout,
            bind_group,
        })
    }

    pub fn update(&mut self, bytes: &[u8], queue: &wgpu::Queue) {
        unimplemented!("need to recalculate");
        /*
        if bytes.len() > self.row_size * 4 {
            panic!("BYTES LONGER THAN TEX SIZE!");
        }
        queue.write_texture(
            wgpu::TextureCopyView {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &bytes,
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: (self.row_size) as u32,
                rows_per_image: self.row_size as u32,
            },
            self.size,
        );
        */
    }
}

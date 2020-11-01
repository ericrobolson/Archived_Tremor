use super::super::{texture::Texture, vertex::Vertex};

use crate::lib_core::{
    ecs::World,
    math::index_3d,
    time::{Clock, Duration, GameFrame, Timer},
    voxels,
};

pub struct Palette {
    palette: voxels::Palette,
    palette_texture: Texture,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl Palette {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let palette = voxels::Palette::new();
        let palette_texture =
            Texture::from_palette(&palette, device, queue, "Palette Texture").unwrap();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::D1,
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
                label: Some("palette_texture_bind_group_layout"),
            });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&palette_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&palette_texture.sampler),
                },
            ],
            label: Some("palette_texture_bind_group"),
        });

        Self {
            palette,
            palette_texture,
            texture_bind_group_layout,
            bind_group,
        }
    }
}

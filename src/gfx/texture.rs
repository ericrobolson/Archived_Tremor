use image::GenericImageView;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    pub fn create_depth_texture(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: sc_desc.width,
            height: sc_desc.height,
            depth: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
        };

        let texture = device.create_texture(&desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }

    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
        is_normal_map: bool,
    ) -> Result<Self, String> {
        let img = match image::load_from_memory(bytes) {
            Ok(i) => i,
            Err(e) => {
                return Err(format!("{:?}", e));
            }
        };

        Self::from_image(device, queue, &img, Some(label), is_normal_map)
    }

    pub fn load<P: AsRef<std::path::Path>>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: P,
        is_normal_map: bool,
    ) -> Result<Self, String> {
        let path_copy = path.as_ref().to_path_buf();
        let label = path_copy.to_str();

        let img = match image::open(path) {
            Ok(i) => i,
            Err(e) => {
                return Err(format!("{:?}", e));
            }
        };

        Self::from_image(device, queue, &img, label, is_normal_map)
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
        is_normal_map: bool,
    ) -> Result<Self, String> {
        let rgba = img.to_rgba();
        let dimensions = img.dimensions();

        let tex_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: dimensions.0,
                height: dimensions.1,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: if is_normal_map {
                wgpu::TextureFormat::Rgba8Unorm
            } else {
                wgpu::TextureFormat::Rgba8UnormSrgb
            },
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            label: Some("diffuse_texture"),
        });

        queue.write_texture(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: 4 * dimensions.0,
                rows_per_image: dimensions.1,
            },
            tex_size,
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

        Ok(Self {
            texture,
            view: texture_view,
            sampler,
        })
    }
}

pub struct Texture3d {
    row_size: usize,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    format: wgpu::TextureFormat,
    size: wgpu::Extent3d,
}

struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}
fn sdf_sphere(point: Vec3) -> f32 {
    let radius = 1.0;

    let len = point.x * point.x + point.y * point.y + point.z * point.z;
    let len = len.sqrt();

    return len - radius;
}

impl Texture3d {
    pub fn new(
        row_size: usize,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: &str,
    ) -> Result<Self, String> {
        let row_size_cubed = row_size * row_size * row_size;
        let mut data = Vec::with_capacity(4 * row_size_cubed); // 4 bytes per item. row_size^3 = total items
        for _ in 0..row_size_cubed {
            data.push(0);
        }

        // Testing simple creation of 3d texture based on a sphere
        let mut floats = Vec::with_capacity(row_size_cubed);
        for z in 0..row_size {
            for y in 0..row_size {
                for x in 0..row_size {
                    let dist = sdf_sphere(Vec3 {
                        x: (x as f32),
                        y: (y as f32) - 1.0,
                        z: (z as f32) - 6.0,
                    });

                    floats.push(dist);
                }
            }
        }

        let data = floats
            .iter()
            .map(|d| d.to_ne_bytes())
            .collect::<Vec<[u8; 4]>>()
            .iter()
            .flat_map(|d| d.iter())
            .map(|d| *d)
            .collect::<Vec<u8>>();

        return Self::from_bytes(
            row_size,
            wgpu::TextureFormat::R32Float,
            device,
            queue,
            &data,
            label,
        );
    }

    pub fn update(&mut self, bytes: &[u8], queue: &wgpu::Queue) {
        unimplemented!("need to recalculate");
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
    }

    pub fn from_bytes(
        row_size: usize,
        format: wgpu::TextureFormat,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self, String> {
        let size = wgpu::Extent3d {
            width: row_size as u32,
            height: row_size as u32,
            depth: row_size as u32,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: format, //wgpu::TextureFormat::R32Float,
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
                bytes_per_row: (4 * row_size) as u32,
                rows_per_image: row_size as u32,
            },
            size,
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
            row_size,
            texture,
            view: texture_view,
            sampler,
            texture_bind_group_layout,
            bind_group,
        })
    }
}

use image::{DynamicImage, GenericImageView};

pub trait Image {
    fn texture(&self) -> &wgpu::Texture;
    fn view(&self) -> &wgpu::TextureView;
    fn sampler(&self) -> &wgpu::Sampler;
}

pub struct TextureAtlas {
    texture: Texture,
    width: u32,
    height: u32,
}

impl TextureAtlas {
    pub fn empty(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        label: Option<&str>,
        filter: bool,
        is_normal_map: bool,
    ) -> Result<Self, String> {
        let img = DynamicImage::new_rgba8(width, height);

        let texture = Texture::from_image(
            device,
            queue,
            &img,
            label,
            filter,
            is_normal_map,
            None,
            None,
        )?;

        Ok(Self {
            texture,
            width,
            height,
        })
    }

    pub fn copy_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        label: Option<&str>,
        filter: bool,
        is_normal_map: bool,
        format_override: wgpu::TextureFormat,
    ) -> Result<Self, String> {
        let img = DynamicImage::new_rgba8(width, height);

        let texture = Texture::from_image(
            device,
            queue,
            &img,
            label,
            filter,
            is_normal_map,
            Some(format_override),
            Some(vec![
                wgpu::TextureUsage::SAMPLED,
                wgpu::TextureUsage::COPY_DST,
                wgpu::TextureUsage::STORAGE,
            ]),
        )?;

        Ok(Self {
            texture,
            width,
            height,
        })
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.texture.view
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture.texture
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn write(&mut self, x: u32, y: u32, img: &image::DynamicImage, queue: &wgpu::Queue) {
        let rgba = img.to_rgba8();
        let (img_width, img_height) = img.dimensions();

        if img_width > self.width || img_height > self.height {
            panic!(
                "Attempted to upload img with size ({}, {}) where atlas size is ({}, {})!",
                img_width, img_height, self.width, self.height
            );
        }

        if img_width + x > self.width || img_height + y > self.height {
            panic!(
                "Attempted to upload img with size ({}, {}) at position ({}, {}), which overflows atlas size of ({}, {})!",
                img_width, img_height, x, y, self.width, self.height
            );
        }

        let tex_size = wgpu::Extent3d {
            width: img_width,
            height: img_height,
            depth: 1,
        };

        let origin = wgpu::Origin3d { x, y, z: 0 };

        queue.write_texture(
            wgpu::TextureCopyView {
                texture: self.texture(),
                mip_level: 0,
                origin: origin,
            },
            &rgba,
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: 4 * img_width,
                rows_per_image: img_height,
            },
            tex_size,
        );
    }
}

impl Image for TextureAtlas {
    fn texture(&self) -> &wgpu::Texture {
        &self.texture.texture
    }
    fn view(&self) -> &wgpu::TextureView {
        &self.texture.view
    }
    fn sampler(&self) -> &wgpu::Sampler {
        &self.texture.sampler
    }
}

pub struct Texture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

impl Image for Texture {
    fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }
    fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
    fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    pub fn create_depth_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
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
        filter: bool,
        is_normal_map: bool,
    ) -> Result<Self, String> {
        let img = match image::load_from_memory(bytes) {
            Ok(i) => i,
            Err(e) => {
                return Err(format!("{:?}", e));
            }
        };

        Self::from_image(
            device,
            queue,
            &img,
            Some(label),
            filter,
            is_normal_map,
            None,
            None,
        )
    }

    pub fn load<P: AsRef<std::path::Path>>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: P,
        filter: bool,
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

        Self::from_image(
            device,
            queue,
            &img,
            label,
            filter,
            is_normal_map,
            None,
            None,
        )
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
        filter: bool,
        is_normal_map: bool,
        format_override: Option<wgpu::TextureFormat>,
        flags: Option<Vec<wgpu::TextureUsage>>,
    ) -> Result<Self, String> {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let tex_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth: 1,
        };

        let format = {
            if is_normal_map {
                wgpu::TextureFormat::Rgba8Unorm
            } else {
                match format_override {
                    Some(format_override) => format_override,
                    None => wgpu::TextureFormat::Rgba8UnormSrgb,
                }
            }
        };

        let usage = {
            let mut usage = wgpu::TextureUsage::empty();
            match flags {
                Some(usages) => {
                    for flag in usages {
                        usage |= flag;
                    }
                }
                None => {
                    usage = wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST;
                }
            }

            usage
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
            format,
            usage,
            label,
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

        let filter = {
            if filter {
                wgpu::FilterMode::Linear
            } else {
                wgpu::FilterMode::Nearest
            }
        };

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: filter,
            min_filter: filter,    //prev wgpu::FilterMode::Nearest
            mipmap_filter: filter, // prev wgpu::FilterMode::Nearest
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
    let radius = 10.1;

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

        let world_size = 50.0;
        for z in 0..row_size {
            for y in 0..row_size {
                for x in 0..row_size {
                    let dist = sdf_sphere(Vec3 {
                        x: (x as f32),
                        y: (y as f32),
                        z: (z as f32),
                    });

                    println!("X: {}, Y: {}, Z: {}, DIST: {}", x, y, z, dist);

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

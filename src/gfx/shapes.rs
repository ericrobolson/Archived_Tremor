use crate::lib_core::ecs::{Mask, MaskType, World};
use crate::lib_core::math::{FixedNumber, Vec3};
use crate::lib_core::shapes::{sphere_sdf, Csg};
/// CSG shapes render pass that writes to the gpu
pub struct ShapesPass {
    pub bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
    max_data: usize,
}

impl ShapesPass {
    pub fn new(device: &wgpu::Device) -> (wgpu::BindGroupLayout, Self) {
        let max_data = 420; // TODO: Update?

        let mut data = Vec::with_capacity(max_data);
        for _ in 0..max_data {
            data.push(0);
        }

        use wgpu::util::DeviceExt;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("csg_buf"),
            contents: &data,
            usage: wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::StorageBuffer {
                    dynamic: false,
                    readonly: true,
                    min_binding_size: wgpu::BufferSize::new(1 as u64), //TODO: fix up?
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(buffer.slice(..)),
            }],
        });

        (
            bind_group_layout,
            Self {
                bind_group,
                buffer,
                max_data: max_data,
            },
        )
    }

    fn create_3d_texture(&self, world: &World) {
        // Example for creating 3d texture: https://community.khronos.org/t/creating-a-3d-texture-from-data/59219
        const SIZE: usize = 4;
        let max_world: FixedNumber = 100.into();
        let min_world: FixedNumber = -max_world;

        let mut data: Vec<f32> = Vec::with_capacity(SIZE * SIZE * SIZE);
        for zi in 0..SIZE {
            let z = translate_to_world_coord(zi, SIZE, min_world, max_world);
            for yi in 0..SIZE {
                let y = translate_to_world_coord(yi, SIZE, min_world, max_world);

                for xi in 0..SIZE {
                    let x = translate_to_world_coord(xi, SIZE, min_world, max_world);
                    let point = Vec3 { x, y, z };
                    let dist: FixedNumber = 0.into(); // TODO: max dist

                    let pos = Vec3::new(); // TODO: get
                    let radius: FixedNumber = 0.into(); // TODO: pull

                    let dist = sphere_sdf(point, pos, radius);
                    data.push(dist.into());
                }
            }
        }
    }

    fn get_buff_data(&self, world: &World) -> Vec<u8> {
        let mut data = vec![];

        const SYS_MASK: MaskType = Mask::POSITION | Mask::SHAPE;
        for entity in world
            .masks
            .iter()
            .enumerate()
            .filter(|(i, mask)| **mask & SYS_MASK == SYS_MASK)
            .map(|(i, mask)| i)
        {
            // TODO: iterate over csgs and whatnot, writing their data to the buffer
            // TODO: interpolate velocities with positions?
            let pos = world.positions[entity];
            let shape = world.shapes[entity];

            match shape {
                Csg::Sphere { radius } => {
                    // Push position
                    let shape_type: f32 = 1.0;
                    let pos: [f32; 3] = pos.into();
                    let radius: f32 = (radius).into();

                    let shape_bytes = shape_type
                        .to_ne_bytes()
                        .iter()
                        .map(|d| *d)
                        .collect::<Vec<u8>>();

                    let pos_bytes = pos
                        .iter()
                        .map(|d| d.to_ne_bytes())
                        .collect::<Vec<[u8; 4]>>()
                        .iter()
                        .flat_map(|d| d.iter())
                        .map(|d| *d)
                        .collect::<Vec<u8>>();

                    let radius_bytes = radius.to_ne_bytes().iter().map(|d| *d).collect::<Vec<u8>>();

                    let mut bytes = shape_bytes
                        .iter()
                        .chain(pos_bytes.iter().chain(radius_bytes.iter()))
                        .map(|d| *d)
                        .collect::<Vec<u8>>();

                    if (data.len() + bytes.len()) < self.max_data {
                        data.append(&mut bytes);
                    }
                }
                _ => {}
            }
        }

        data
    }

    pub fn update(&mut self, queue: &wgpu::Queue, world: &World) {
        let data = self.get_buff_data(world);
        queue.write_buffer(&self.buffer, 0, &data);
    }
}

fn translate_to_world_coord(
    index: usize,
    max_indices: usize,
    min_world: FixedNumber,
    max_world: FixedNumber,
) -> FixedNumber {
    unimplemented!()
}

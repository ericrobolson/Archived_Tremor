use crate::lib_core::{
    ecs::{Mask, MaskType, World},
    math::{FixedNumber, Vec3},
    shapes::{sphere_sdf, Csg},
    time::{Clock, Timer},
};
/// CSG shapes render pass that writes to the gpu
pub struct ShapesPass {
    pub bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
    max_data: usize,
    timer: Timer,
}

const ROW_SIZE: usize = 4; // row size for 3d texture

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
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None, //min_binding_size: wgpu::BufferSize::new(1 as u64), //TODO: fix up?
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
                timer: Timer::new(60),
            },
        )
    }

    fn tex3d(device: &wgpu::Device, queue: &wgpu::Queue) {}

    fn create_3d_texture(&mut self, world: &World) {
        //TODO: upload to gpu and run in compute shader?
        if self.timer.can_run() {
            let mut clock = Clock::new();

            // Example for creating 3d texture: https://community.khronos.org/t/creating-a-3d-texture-from-data/59219
            const SIZE: usize = ROW_SIZE;
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
                        let mut dist: FixedNumber = 1000.into(); // TODO: max dist

                        const SYS_MASK: MaskType = Mask::POSITION | Mask::SHAPE;
                        for entity in world
                            .masks
                            .iter()
                            .enumerate()
                            .filter(|(i, mask)| **mask & SYS_MASK == SYS_MASK)
                            .map(|(i, mask)| i)
                        {
                            let pos = world.positions[entity];
                            let shape = world.shapes[entity];

                            match shape {
                                Csg::Sphere { radius } => {
                                    dist = FixedNumber::min(sphere_sdf(point, pos, radius), dist);
                                }
                                _ => {}
                            }
                        }

                        data.push(dist.into());
                    }
                }
            }

            let duration = clock.stop_watch();
            println!("Frame time: {:?}", duration);
        }
    }

    fn get_buff_data(&mut self, world: &World) -> Vec<u8> {
        //self.create_3d_texture(world);
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
    if index == 0 {
        return min_world;
    } else if index == (max_indices - 1) {
        return max_world;
    }

    if max_world == min_world {
        return min_world;
    }

    // Rough version. May requires some more indepth thinking.

    let max_indices: FixedNumber = (max_indices - 1).into();
    let index: FixedNumber = index.into();

    let scale = (max_world - min_world) / (max_indices);
    let offset = min_world;
    return index * scale + offset;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_to_world_coord_maps() {
        let index = 0;
        let max_indices = 4;
        let max: FixedNumber = 100.into();
        let min: FixedNumber = -max;

        let expected: FixedNumber = min;
        let actual: FixedNumber = translate_to_world_coord(index, max_indices, min, max);
        assert_eq!(expected, actual);

        let index = 3;
        let max_indices = 4;
        let max: FixedNumber = 100.into();
        let min: FixedNumber = -max;

        let expected: FixedNumber = max;
        let actual: FixedNumber = translate_to_world_coord(index, max_indices, min, max);
        assert_eq!(expected, actual);

        let index = 1;
        let max_indices = 4;
        let max: FixedNumber = 100.into();
        let min: FixedNumber = -max;

        let expected: FixedNumber = max;
        let actual: FixedNumber = translate_to_world_coord(index, max_indices, min, max);
        assert_eq!(expected, actual);
    }
}

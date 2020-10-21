use crate::lib_core::ecs::{Mask, MaskType, World};

/// CSG shapes render pass that writes to the gpu
pub struct ShapesPass {
    pub bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
}

impl ShapesPass {
    pub fn new(device: &wgpu::Device) -> (wgpu::BindGroupLayout, Self) {
        use wgpu::util::DeviceExt;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("csg_buf"),
            contents: &[],
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
                    min_binding_size: wgpu::BufferSize::new(0), //TODO: fix up?
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

        (bind_group_layout, Self { bind_group, buffer })
    }

    pub fn update(&mut self, queue: &wgpu::Queue, world: &World) {
        let mut data = vec![];

        const SYS_MASK: MaskType = Mask::POSITION & Mask::SHAPES;
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
            let shape = &world.shapes[entity];
        }

        queue.write_buffer(&self.buffer, 0, &data);
    }
}

use std::marker::PhantomData;

use crate::gfx::DeviceQueue;
use wgpu::util::DeviceExt;

pub trait ToGpuRaw<Raw> {
    /// Converts the type to a raw GPU representation.
    fn to_gpu_raw(&self) -> Raw
    where
        Raw: bytemuck::Pod + bytemuck::Zeroable;
}

fn init_uniform_buffer_bind_group<U>(
    uniforms: U,
    uniform_bind_group_layout: &wgpu::BindGroupLayout,
    dq: &DeviceQueue,
) -> (wgpu::BindGroup, wgpu::Buffer)
where
    U: bytemuck::Pod + bytemuck::Zeroable,
{
    let uniform_buffer = dq
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

    let uniform_bind_group = dq.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &uniform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(uniform_buffer.slice(..)),
        }],
        label: Some("uniform_bind_group"),
    });

    (uniform_bind_group, uniform_buffer)
}

pub struct UniformContainer<U, Raw>
where
    U: ToGpuRaw<Raw>,
    Raw: bytemuck::Pod + bytemuck::Zeroable,
{
    uniform: U,
    bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
    dirty: bool,
    phantom: PhantomData<Raw>,
}

impl<U, Raw> UniformContainer<U, Raw>
where
    U: ToGpuRaw<Raw>,
    Raw: bytemuck::Pod + bytemuck::Zeroable,
{
    /// Creates a BindGroupLayout for the given uniform.
    pub fn init_bindgroup_layout(
        name: &'static str,
        device: &wgpu::Device,
        visibility: wgpu::ShaderStage,
        binding: Option<u32>,
    ) -> wgpu::BindGroupLayout {
        let binding = binding.unwrap_or(0);

        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding,
                visibility,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some(name),
        })
    }

    /// Creates a new UniformContainer.
    pub fn new(uniform: U, layout: &wgpu::BindGroupLayout, dq: &DeviceQueue) -> Self {
        let (bind_group, buffer) =
            init_uniform_buffer_bind_group(uniform.to_gpu_raw(), &layout, dq);

        Self {
            uniform,
            bind_group,
            buffer,
            dirty: false,
            phantom: PhantomData,
        }
    }

    /// Returns a reference to the uniform
    pub fn uniform(&self) -> &U {
        &self.uniform
    }

    /// Returns a mutable reference to the uniform.
    pub fn uniform_mut(&mut self) -> &mut U {
        self.dirty = true;
        &mut self.uniform
    }

    /// Returns the uniform's bindgroup.
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    /// Writes the uniform to the GPU buffer.
    pub fn write_buffer(&mut self, dq: &DeviceQueue) {
        if self.dirty {
            self.dirty = false;
            dq.queue.write_buffer(
                &self.buffer,
                0,
                bytemuck::cast_slice(&[self.uniform.to_gpu_raw()]),
            );
        }
    }
}

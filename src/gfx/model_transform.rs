use crate::lib_core::spatial;

use super::conversions::*;

/// A combined matrix for position, scale and rotation
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ModelTransform {
    transformation: cgmath::Matrix4<f32>,
}

unsafe impl bytemuck::Pod for ModelTransform {}
unsafe impl bytemuck::Zeroable for ModelTransform {}

impl ModelTransform {
    pub fn new(transform: spatial::Transformation) -> Self {
        Self {
            transformation: transformation_matrix(transform),
        }
    }

    pub fn init_buffers(
        device: &wgpu::Device,
    ) -> (wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup) {
        use wgpu::util::DeviceExt;

        let model_transform = spatial::Transformation::default();
        let model_transform = Self::new(model_transform);

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Model Transform Buffer"),
            contents: bytemuck::cast_slice(&[model_transform]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("model_transform_bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(buffer.slice(..)),
            }],
            label: Some("model_transform_bind_group"),
        });

        (buffer, bind_group_layout, bind_group)
    }
}

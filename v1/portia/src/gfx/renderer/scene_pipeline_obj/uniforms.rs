use crate::gfx::{DeviceQueue, DoubleBuffer};

use rendering_ir::{camera3d::Camera3d, wgpu_helpers::OPENGL_TO_WGPU_MATRIX};

use cgmath::prelude::*;
use wgpu::util::DeviceExt;

#[repr(C)] // We need this for Rust to store our data correctly for the shaders
#[derive(Debug, Copy, Clone)] // This is so we can store this in a buffer
pub struct Uniforms {
    pub view_proj: [[f32; 4]; 4],
    pub view_pos: [f32; 3],
    pub view: [[f32; 4]; 4],
}

impl Uniforms {
    fn new(camera: &Camera3d) -> Self {
        let mut u = Self {
            view_proj: cgmath::Matrix4::identity().into(),
            view_pos: camera.eye(),
            view: camera.view_matrix().into(),
        };

        u.update_view_proj(camera);
        u
    }

    fn update_screen_size(&mut self, width: f32, height: f32) {
        //self.screen_size = (width, height).into();
    }

    fn update_view_proj(&mut self, camera: &Camera3d) {
        //proj * view
        let proj = camera.projection_matrix();
        let view = camera.view_matrix();

        self.view_proj = (OPENGL_TO_WGPU_MATRIX * proj * view).into();
        self.view_pos = camera.eye();
        self.view = view.into();
    }
}

unsafe impl bytemuck::Pod for Uniforms {}
unsafe impl bytemuck::Zeroable for Uniforms {}

/// Container to manage uniforms. Utilizes double buffering.
pub struct UniformContainer {
    double_buffer: DoubleBuffer,
    uniforms_a: Uniforms,
    bind_group_a: wgpu::BindGroup,
    buffer_a: wgpu::Buffer,
    uniforms_b: Uniforms,
    bind_group_b: wgpu::BindGroup,
    buffer_b: wgpu::Buffer,
}

fn init_buffer_bind_group(
    uniforms: Uniforms,
    uniform_bind_group_layout: &wgpu::BindGroupLayout,
    dq: &DeviceQueue,
) -> (wgpu::BindGroup, wgpu::Buffer) {
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

impl UniformContainer {
    pub fn new(
        camera: &Camera3d,
        uniform_bind_group_layout: &wgpu::BindGroupLayout,
        dq: &DeviceQueue,
    ) -> Self {
        let uniforms_a = Uniforms::new(camera);
        let uniforms_b = Uniforms::new(camera);

        let (bind_group_a, buffer_a) =
            init_buffer_bind_group(uniforms_a, uniform_bind_group_layout, dq);

        let (bind_group_b, buffer_b) =
            init_buffer_bind_group(uniforms_b, uniform_bind_group_layout, dq);

        Self {
            double_buffer: DoubleBuffer::UpdateARenderB,
            uniforms_a: Uniforms::new(camera),
            bind_group_a,
            buffer_a,
            uniforms_b: Uniforms::new(camera),
            bind_group_b,
            buffer_b,
        }
    }

    pub fn update_buffer(&mut self, dq: &DeviceQueue) {
        match self.double_buffer {
            DoubleBuffer::UpdateARenderB => {
                dq.queue
                    .write_buffer(&self.buffer_a, 0, bytemuck::cast_slice(&[self.uniforms_a]));
            }
            DoubleBuffer::UpdateBRenderA => {
                dq.queue
                    .write_buffer(&self.buffer_b, 0, bytemuck::cast_slice(&[self.uniforms_b]));
            }
        }
    }

    /// Returns the render buffer
    pub fn buffer(&self) -> &wgpu::Buffer {
        match self.double_buffer {
            DoubleBuffer::UpdateARenderB => &self.buffer_b,
            DoubleBuffer::UpdateBRenderA => &self.buffer_a,
        }
    }

    /// Returns the render bind group
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        match self.double_buffer {
            DoubleBuffer::UpdateARenderB => &self.bind_group_b,
            DoubleBuffer::UpdateBRenderA => &self.bind_group_a,
        }
    }

    pub fn update_screen_size(&mut self, width: f32, height: f32) {
        //self.screen_size = (width, height).into();
    }

    pub fn update(&mut self, camera: &Camera3d) {
        match self.double_buffer {
            DoubleBuffer::UpdateARenderB => {
                self.double_buffer = DoubleBuffer::UpdateBRenderA;
                self.uniforms_b.update_view_proj(camera);
            }
            DoubleBuffer::UpdateBRenderA => {
                self.double_buffer = DoubleBuffer::UpdateARenderB;
                self.uniforms_a.update_view_proj(camera);
            }
        }
    }
}

fn u32_f32(v: (u32, u32)) -> (f32, f32) {
    (v.0 as f32, v.1 as f32)
}

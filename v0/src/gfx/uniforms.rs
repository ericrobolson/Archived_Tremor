use crate::gfx::camera::Camera;
use cgmath::prelude::*;

#[repr(C)] // We need this for Rust to store our data correctly for the shaders
#[derive(Debug, Copy, Clone)] // This is so we can store this in a buffer
pub struct Uniforms {
    pub view_position: cgmath::Vector4<f32>,
    pub view_proj: cgmath::Matrix4<f32>,
    pub viewport_size: cgmath::Vector2<f32>,
}

unsafe impl bytemuck::Pod for Uniforms {}
unsafe impl bytemuck::Zeroable for Uniforms {}

impl Uniforms {
    pub fn new(viewport_x: f32, viewport_y: f32) -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_position: Zero::zero(),
            view_proj: cgmath::Matrix4::identity(),
            viewport_size: (viewport_x, viewport_y).into(),
        }
    }

    pub fn update_viewport_size(&mut self, width: f32, height: f32) {
        self.viewport_size = (width, height).into();
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_position = camera.eye.to_homogeneous();
        self.view_proj = camera.build_view_projection_matrix();
    }
}

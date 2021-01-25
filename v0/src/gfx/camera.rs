use crate::gfx::OPENGL_TO_WGPU_MATRIX;
use cgmath::{perspective, Matrix4, Point3, Vector3};

use crate::lib_core::{ecs::World, spatial};

pub struct Camera {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn from_camera(camera: &spatial::Camera, screen_width: f32, screen_height: f32) -> Self {
        let mut cam = Self {
            eye: camera.eye.into(),
            target: camera.target.into(),
            up: camera.up.into(),
            aspect: 0.0,
            fovy: camera.fovy.into(),
            znear: camera.znear.into(),
            zfar: camera.zfar.into(),
        };

        cam.resize(screen_width, screen_height);

        cam
    }

    pub fn resize(&mut self, screen_width: f32, screen_height: f32) {
        self.aspect = screen_width / screen_height;
    }

    pub fn update(&mut self, world: &World) {
        self.eye = world.camera.eye.into();
        self.target = world.camera.target.into();
        self.up = world.camera.up.into();
        self.fovy = world.camera.fovy.into();
        self.znear = world.camera.znear.into();
        self.zfar = world.camera.zfar.into();
    }

    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::look_at(self.eye, self.target, self.up);
        let proj = perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

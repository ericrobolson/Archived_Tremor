use cgmath::{ortho, perspective, Matrix4, Point3, Vector3};

#[derive(Copy, Clone, Debug, PartialEq)]
/// 3d camera
pub struct Camera3d {
    eye: Point3<f32>,
    target: Point3<f32>,
    up: Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
    orthographic: bool,
    screen_width: f32,
    screen_height: f32,
}

impl Camera3d {
    /// Creates a new 3d camera.
    pub fn new(
        screen_width: f32,
        screen_height: f32,
        target: [f32; 3],
        eye: [f32; 3],
        orthographic: bool,
    ) -> Self {
        let mut cam = Self {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: screen_width / screen_height,
            fovy: 45.0,
            znear: 0.1,
            zfar: 10000.0,
            screen_width,
            screen_height,
            orthographic,
        };

        cam.resize(screen_width, screen_height);
        cam.update(target, eye, orthographic);
        cam
    }

    /// The field of view in the y, degrees
    pub fn fovy(&self) -> f32 {
        self.fovy
    }

    /// Resize the camera
    pub fn resize(&mut self, screen_width: f32, screen_height: f32) {
        self.aspect = screen_width / screen_height;
        self.screen_height = screen_height;
        self.screen_width = screen_width;
    }

    /// Update the camera
    pub fn update(&mut self, target: [f32; 3], eye: [f32; 3], orthographic: bool) {
        self.orthographic = orthographic;
        self.eye = eye.into();
        self.target = target.into();
    }

    /// Get the eye of the camera
    pub fn eye(&self) -> [f32; 3] {
        self.eye.into()
    }

    /// Returns the view matrix of the camera
    pub fn view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(self.eye, self.target, self.up)
    }

    /// Returns the projection matrix of the camera
    pub fn projection_matrix(&self) -> Matrix4<f32> {
        if self.orthographic {
            let w = 1.;
            let h = 1. / self.aspect;

            ortho(-w, w, -h, h, self.znear, self.zfar)
        } else {
            perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar)
        }
    }
}

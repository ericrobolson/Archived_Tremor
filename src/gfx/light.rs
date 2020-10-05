use cgmath::Vector3;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Light {
    pub position: Vector3<f32>,
    // padding for uniforms 16 byte len (4 floats)
    pub _padding: u32,
    pub color: Vector3<f32>,
}

unsafe impl bytemuck::Zeroable for Light {}
unsafe impl bytemuck::Pod for Light {}

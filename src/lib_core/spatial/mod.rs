use crate::lib_core::math::{FixedNumber, Vec3};

mod transform;
pub use transform::Transform;

pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fovy: FixedNumber,
    pub znear: FixedNumber,
    pub zfar: FixedNumber,
}

impl Camera {
    pub fn new(eye: Vec3, target: Vec3) -> Self {
        Self {
            eye,
            target,
            up: Vec3::unit_y(),
            fovy: 45.into(),
            znear: FixedNumber::fraction(10.into()),
            zfar: 1000.into(),
        }
    }
}

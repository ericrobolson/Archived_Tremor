use crate::lib_core::math::{FixedNumber, Vec3};

pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fovy: FixedNumber,
    pub znear: FixedNumber,
    pub zfar: FixedNumber,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            eye: (0, 1, -10).into(),
            target: (0, 0, 0).into(),
            up: Vec3::unit_y(),
            fovy: 45.into(),
            znear: FixedNumber::fraction(10.into()),
            zfar: 1000.into(),
        }
    }
}

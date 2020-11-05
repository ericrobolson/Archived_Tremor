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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PhysicBodies {
    Kinematic,
    Static,
    Rigidbody,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn new() -> Self {
        Self {
            min: Vec3::new(),
            max: Vec3::new(),
        }
    }

    pub fn colliding(
        &self,
        transform: &Transform,
        other: &Self,
        other_transform: &Transform,
    ) -> bool {
        //TODO: rotations
        let a_min = transform.position + self.min;
        let a_max = transform.position + self.max;
        let b_min = other_transform.position + other.min;
        let b_max = other_transform.position + other.max;

        return (a_min.x <= b_max.x && a_max.x >= b_min.x)
            && (a_min.y <= b_max.y && a_max.y >= b_min.y)
            && (a_min.z <= b_max.z && a_max.z >= b_min.z);
    }
}

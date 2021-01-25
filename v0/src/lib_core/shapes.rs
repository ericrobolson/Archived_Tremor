use crate::lib_core::math::{FixedNumber, Vec3};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Csg {
    Sphere { radius: FixedNumber },
    Rectangle,
}

pub enum Ops {
    Union,
    Difference,
    Intersection,
}

pub fn sphere_sdf(point: Vec3, sphere_pos: Vec3, radius: FixedNumber) -> FixedNumber {
    (point - sphere_pos).len() - radius
}

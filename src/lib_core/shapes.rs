use crate::lib_core::math::FixedNumber;

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

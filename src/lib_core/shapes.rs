use crate::lib_core::math::FixedNumber;

pub enum Csg {
    Sphere { radius: FixedNumber },
    Rectangle,
}

pub enum Ops {
    Union,
    Difference,
    Intersection,
}

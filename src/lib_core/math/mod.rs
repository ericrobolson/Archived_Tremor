pub mod fixed_number;
pub use fixed_number::FixedNumber;

pub enum Ops {
    Add,
    Subtract,
}

pub fn wrap_op_usize(a: usize, b: usize, op: Ops) -> usize {
    match op {
        Ops::Add => a.wrapping_add(b),
        Ops::Subtract => a.wrapping_sub(b),
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Vec3 {
    pub x: FixedNumber,
    pub y: FixedNumber,
    pub z: FixedNumber,
}

impl Vec3 {
    pub fn new() -> Self {
        Self {
            x: 0.into(),
            y: 0.into(),
            z: 0.into(),
        }
    }
}

impl Into<[f32; 3]> for Vec3 {
    fn into(self) -> [f32; 3] {
        [self.x.into(), self.y.into(), self.z.into()]
    }
}

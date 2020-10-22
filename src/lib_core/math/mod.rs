pub mod fixed_number;
pub use fixed_number::FixedNumber;

pub mod vec3;
pub use vec3::Vec3;

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

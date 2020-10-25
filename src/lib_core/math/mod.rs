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

pub fn index_1d(x: usize, y: usize, z: usize, x_max: usize, y_max: usize, z_max: usize) -> usize {
    let x = x % x_max;
    let y = y % y_max;
    let z = z % z_max;

    x + y * x_max + z * x_max * y_max
}

pub fn index_3d(i: usize, x_max: usize, y_max: usize, z_max: usize) -> (usize, usize, usize) {
    let z = i / (x_max * y_max);
    let i = i - (z * x_max * y_max);
    let y = i / x_max;
    let x = i % x_max;

    (x, y, z)
}

pub mod fixed_number;
pub use fixed_number::FixedNumber;
pub use fixed_number::FixedNumberLut;

pub mod vec3;
pub use vec3::Vec3;
pub mod quaternion;
pub use quaternion::Quaternion;

pub enum Ops {
    Add,
    Subtract,
}

pub fn rng(max: u32) -> u32 {
    // TODO: make deterministic
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let num: u32 = rng.gen();

    num % max
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

pub fn index_2d_to_1d(x: usize, y: usize, x_depth: usize, y_depth: usize) -> usize {
    let x = x % x_depth;
    let y = y % y_depth;
    x_depth * x + y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn Math_index_1d_works_as_expected() {
        let x_depth = 3;
        let y_depth = 4;
        let z_depth = 5;

        let (x, y, z) = (0, 0, 0);
        let expected = 0;
        let actual = index_1d(x, y, z, x_depth, y_depth, z_depth);
        assert_eq!(expected, actual);

        let (x, y, z) = (1, 2, 3);
        let expected = x + y * x_depth + z * x_depth * y_depth;
        let actual = index_1d(x, y, z, x_depth, y_depth, z_depth);
        assert_eq!(expected, actual);

        // Boundary check
        let (x, y, z) = (x_depth, y_depth, z_depth);
        let expected = 0;
        let actual = index_1d(x, y, z, x_depth, y_depth, z_depth);
        assert_eq!(expected, actual);
    }

    #[test]
    fn Math_index_3d_works_as_expected() {
        let x_depth = 3;
        let y_depth = 4;
        let z_depth = 5;

        let (x, y, z) = (1, 2, 3);
        let expected = (x, y, z);
        let i = index_1d(x, y, z, x_depth, y_depth, z_depth);
        let actual = index_3d(i, x_depth, y_depth, z_depth);
        assert_eq!(expected, actual);

        let (x, y, z) = (3, 3, 2);
        let expected = (0, y, z);
        let i = index_1d(x, y, z, x_depth, y_depth, z_depth);
        let actual = index_3d(i, x_depth, y_depth, z_depth);
        assert_eq!(expected, actual);
    }
}

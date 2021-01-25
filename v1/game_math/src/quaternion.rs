use crate::{mat4::Mat4, vec4::Vec4};

use super::vec3::Vec3;
use super::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Quaternion<R>
where
    R: Number,
{
    // Scalar
    w: R,
    x: R,
    y: R,
    z: R,
}

// Derived from: https://www.cprogramming.com/tutorial/3d/quaternions.html
// Derived from: https://github.com/MartinWeigel/Quaternion/blob/master/Quaternion.c

impl<R> Quaternion<R>
where
    R: Number,
{
    /// Creates a new Quaternion from the specified values.
    pub fn new(x: R, y: R, z: R, w: R) -> Self {
        Self {
            w,
            x,
            y,
            z,
        }
    }

    /// The identity of a Quaternion.
    pub fn identity() -> Self {
        Self::new(R::i32(0), R::i32(0), R::i32(0), R::i32(1))
    }

    fn from_axis_angle(axis: Vec3<R>, angle: R) -> Self {
        let angle = angle / R::i32(2);

        let w = angle.cos();
        let c = angle.sin();

        Self::new( c * axis.x, c * axis.y, c * axis.z, w)
    }

    pub fn from_x_rotation(angle: R) -> Self {
        Self::from_axis_angle((1, 0, 0).into(), angle)
    }

    pub fn from_y_rotation(angle: R) -> Self {
        Self::from_axis_angle((0, 1, 0).into(), angle)
    }

    pub fn from_z_rotation(angle: R) -> Self {
        Self::from_axis_angle((0, 0, 1).into(), angle)
    }

    /// Normalizes the Quaternion, setting its length to 1.
    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag == R::i32(1) {
            return *self;
        }

        let mag = mag.sqrt();
        let w = self.w / mag;
        let x = self.x / mag;
        let y = self.y / mag;
        let z = self.z / mag;

        Self::new(x, y, z, w)
    }

    /// Converts the Quaternion to a Mat4.
    pub fn to_mat4(&self) -> Mat4<R>{
        let m = self.to_matrix();
        let mut i = 0;
        let m0 = Vec4::new(m[i][0], m[i][1], m[i][2], m[i][3]);
        i += 1;
        let m1 = Vec4::new(m[i][0], m[i][1], m[i][2], m[i][3]);
        i += 1;
        let m2 = Vec4::new(m[i][0], m[i][1], m[i][2], m[i][3]);
        i += 1;
        let m3 = Vec4::new(m[i][0], m[i][1], m[i][2], m[i][3]);
        
        Mat4::new(m0, m1, m2, m3)
    }

    fn to_matrix(&self) -> [[R; 4]; 4] {
        let zero = R::i32(0);
        let one = R::i32(1);
        let two: R = R::i32(2);

        let w = self.w;
        let x = self.x;
        let y = self.y;
        let z = self.z;

        let x_sqrd = x.sqrd();
        let y_sqrd = y.sqrd();
        let z_sqrd = z.sqrd();

        let two_x2 = two * x_sqrd;
        let two_y2 = two * y_sqrd;
        let two_z2 = two * z_sqrd;

        let two_xy = two * x * y;
        let two_xz = two * x * z;
        let two_yz = two * y * z;
        let two_wx = two * w * x;
        let two_wz = two * w * z;
        let two_wy = two * w * y;

        // Since it's normalized, we can shortcut w^2 with 1

        let m1 = [
            one - two_y2 - two_z2,
            two_xy - two_wz,
            two_xz + two_wy,
            zero,
        ];

        let m2 = [
            two_xy + two_wz,
            one - two_x2 - two_z2,
            two_yz + two_wx,
            zero,
        ];

        let m3 = [
            two_xz - two_wy,
            two_yz - two_wx,
            one - two_x2 - two_y2,
            zero,
        ];

        let m4 = [zero, zero, zero, one];

        [m1, m2, m3, m4]
    }

    pub fn rotate_vec3(&self, v: Vec3<R>) -> Vec3<R> {
        // https://gamedev.stackexchange.com/a/50545

        let u = Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        };
        let s = self.w;

        let two: R = R::i32(2);

        let v2 = u * two * u.dot(v) + v * (s.sqrd() - u.dot(u)) + u.cross(v) * two * s;

        v2
    }

    // Multiply two Quaternions. Not commutative, meaning q1 * q2 != q2 * q1.
    fn multiply(&self, other: Self) -> Self {
        let w = self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z;
        let x = self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y;
        let y = self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x;
        let z = self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w;

        // Check if we need to renormalize it?
        Self::new(x, y, z, w)
    }

    fn magnitude(&self) -> R {
        self.w.sqrd() + self.x.sqrd() + self.y.sqrd() + self.z.sqrd()
    }

    /*
    fn should_normalize(&self) -> bool {
        unimplemented!();
        /*
        let tolerance = R::decimal_resolution_value() * R::from_i32(10); // Tolerance is how much rounding errors we can tolerate
        let norm = self.magnitude();

        if R::i32(1) - tolerance < norm && R::i32(1) + tolerance > norm {
            return false;
        }

        true
        */
    }
    */
}

impl<R> std::ops::Mul for Quaternion<R>
where
    R: Number,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Quaternion<R> {
        self.multiply(rhs)
    }
}

impl<R> std::ops::MulAssign for Quaternion<R>
where
    R: Number,
{
    fn mul_assign(&mut self, rhs: Self) {
        let m = self.multiply(rhs);
        self.w = m.w;
        self.x = m.x;
        self.y = m.y;
        self.z = m.z;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Q = Quaternion<f32>;

    #[test]
    fn Quaternion_to_matrix() {
        assert_eq!(true, false);
    }

    #[test]
    fn Quaternion_normalize() {
        let q = Q::from_x_rotation(1.0 / 3.0);

        println!("Q: {:?}", q);

        let expected = {
            println!("MAG: {:?}", q.magnitude());
            let mag = q.magnitude().sqrt();
            let w = q.w / mag;
            let x = q.x / mag;
            let y = q.y / mag;
            let z = q.z / mag;

            Quaternion::new(x, y, z, w)
        };

        assert_eq!(expected, q.normalize());
    }

    #[test]
    fn Quaternion_magnitude() {
        let q = Q::from_x_rotation(1.0 / 3.0);

        let expected = q.w.sqrd() + q.x.sqrd() + q.y.sqrd() + q.z.sqrd();
        let actual = q.magnitude();

        assert_eq!(expected, actual);

        let q = Q::from_z_rotation(1.0 / 3.0);

        let expected = q.w.sqrd() + q.x.sqrd() + q.y.sqrd() + q.z.sqrd();
        let actual = q.magnitude();

        assert_eq!(expected, actual);
    }

    #[test]
    fn Quaternion_mul_assign() {
        let mut q1 = Q::from_x_rotation(1.0 / 3.0);
        let other = Q::from_y_rotation(4.0);

        let expected = q1.multiply(other);
        q1 *= other;

        assert_eq!(expected, q1);

        let mut q1 = Q::from_z_rotation(1.0 / 3.0);
        let other = Q::from_y_rotation(4.0);

        let expected = q1.multiply(other);
        q1 *= other;

        assert_eq!(expected, q1);
    }
    #[test]
    fn Quaternion_mul() {
        let q1 = Q::from_x_rotation(1.0 / 3.0);
        let other = Q::from_y_rotation(4.0);

        let expected = q1.multiply(other);
        let actual = q1 * other;

        assert_eq!(expected, actual);

        let mut q1 = Q::from_z_rotation(1.0 / 3.0);
        let other = Q::from_x_rotation(4.0);

        let expected = q1.multiply(other);
        let actual = q1 * other;

        assert_eq!(expected, actual);
    }

    #[test]
    fn Quaternion_multiply() {
        let q1 = Q::from_x_rotation(1.0 / 3.0);
        let other = Q::from_y_rotation(4.0);

        let w = q1.w * other.w - q1.x * other.x - q1.y * other.y - q1.z * other.z;
        let x = q1.w * other.x + q1.x * other.w + q1.y * other.z - q1.z * other.y;
        let y = q1.w * other.y - q1.x * other.z + q1.y * other.w + q1.z * other.x;
        let z = q1.w * other.z + q1.x * other.y - q1.y * other.x + q1.z * other.w;

        let expected = Q::new(x, y, z, w);
        let actual = q1.multiply(other);
        assert_eq!(expected, actual);

        let q1 = Q::from_z_rotation(1.0 / 3.0);
        let other = Quaternion::from_x_rotation(44.0);

        let w = q1.w * other.w - q1.x * other.x - q1.y * other.y - q1.z * other.z;
        let x = q1.w * other.x + q1.x * other.w + q1.y * other.z - q1.z * other.y;
        let y = q1.w * other.y - q1.x * other.z + q1.y * other.w + q1.z * other.x;
        let z = q1.w * other.z + q1.x * other.y - q1.y * other.x + q1.z * other.w;

        let expected = Q::new(x, y, z, w);
        let actual = q1.multiply(other);
        assert_eq!(expected, actual);
    }

    #[test]
    fn Quaternion_from_z_rotation() {
        let angle = 1.0 / 7.0;

        let expected = Q::from_axis_angle((0, 0, 1).into(), angle);
        let actual = Quaternion::from_z_rotation(angle);
        assert_eq!(expected, actual);

        let angle = 1.0 / 37.0;

        let expected = Q::from_axis_angle((0, 0, 1).into(), angle);
        let actual = Q::from_z_rotation(angle);
        assert_eq!(expected, actual);

        let angle = -3.0;

        let expected = Q::from_axis_angle((0, 0, 1).into(), angle);
        let actual = Q::from_z_rotation(angle);
        assert_eq!(expected, actual);

        let angle = 3.0;

        let expected = Q::from_axis_angle((0, 0, 1).into(), angle);
        let actual = Q::from_z_rotation(angle);
        assert_eq!(expected, actual);
    }

    #[test]
    fn Quaternion_from_y_rotation() {
        let angle = 1.0 / 7.0;

        let expected = Q::from_axis_angle((0, 1, 0).into(), angle);
        let actual = Q::from_y_rotation(angle);
        assert_eq!(expected, actual);

        let angle = 1.0 / 37.0;

        let expected = Q::from_axis_angle((0, 1, 0).into(), angle);
        let actual = Q::from_y_rotation(angle);
        assert_eq!(expected, actual);

        let angle = -3.0;

        let expected = Q::from_axis_angle((0, 1, 0).into(), angle);
        let actual = Q::from_y_rotation(angle);
        assert_eq!(expected, actual);

        let angle = 3.0;

        let expected = Q::from_axis_angle((0, 1, 0).into(), angle);
        let actual = Q::from_y_rotation(angle);
        assert_eq!(expected, actual);
    }

    #[test]
    fn Quaternion_from_x_rotation() {
        let angle = 1.0 / 7.0;

        let expected = Q::from_axis_angle((1, 0, 0).into(), angle);
        let actual = Q::from_x_rotation(angle);
        assert_eq!(expected, actual);

        let angle = 1.0 / 37.0;

        let expected = Q::from_axis_angle((1, 0, 0).into(), angle);
        let actual = Q::from_x_rotation(angle);
        assert_eq!(expected, actual);

        let angle = -3.0;

        let expected = Q::from_axis_angle((1, 0, 0).into(), angle);
        let actual = Q::from_x_rotation(angle);
        assert_eq!(expected, actual);

        let angle = 3.0;

        let expected = Q::from_axis_angle((1, 0, 0).into(), angle);
        let actual = Q::from_x_rotation(angle);
        assert_eq!(expected, actual);
    }

    #[test]
    fn Quaternion_from_axis_angle() {
        let axis: Vec3<f32> = (0, 2, 3).into();
        let angle = 1.0 / 7.0;

        let w = (angle / 2.0).cos();
        let c = (angle / 2.0).sin();

        let expected = Q::new(c * axis.x, c * axis.y, c * axis.z, w);
        let actual = Q::from_axis_angle(axis, angle);

        assert_eq!(expected, actual);

        let axis: Vec3<f32> = (4, 99, 32).into();
        let angle = 1.0 / 133.0;

        let w = (angle / 2.0).cos();
        let c = (angle / 2.0).sin();

        let expected = Q::new(c * axis.x, c * axis.y, c * axis.z, w);
        let actual = Q::from_axis_angle(axis, angle);

        assert_eq!(expected, actual);
    }
    #[test]
    fn Quaternion_new_sets_as_expected() {
        let w = 3.0;
        let v0 = 4.0;
        let v1 = 5.0;
        let v2 = 6.0;

        let q = Q::new(v0, v1, v2, w);

        assert_eq!(w, q.w);
        assert_eq!(v0, q.x);
        assert_eq!(v1, q.y);
        assert_eq!(v2, q.z);
    }

    #[test]
    fn Quaternion_identity_sets_as_expected() {
        let w = 1.0;
        let v0 = 0.0;
        let v1 = 0.0;
        let v2 = 0.0;

        let q = Q::identity();

        assert_eq!(w, q.w);
        assert_eq!(v0, q.x);
        assert_eq!(v1, q.y);
        assert_eq!(v2, q.z);

        let q1 = Q::new(v0, v1, v2, w);

        assert_eq!(q1, q);
    }
}

use super::*;

type R = FixedNumber;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Quaternion {
    // Scalar
    w: R,
    x: R,
    y: R,
    z: R,
}

// Derived from: https://www.cprogramming.com/tutorial/3d/quaternions.html
// Derived from: https://github.com/MartinWeigel/Quaternion/blob/master/Quaternion.c

fn sin(f: R) -> R {
    FixedNumberLut::sin(f)
}
fn cos(f: R) -> R {
    FixedNumberLut::cos(f)
}

impl Quaternion {
    pub fn default() -> Self {
        Self::identity()
    }

    fn new(w: R, v0: R, v1: R, v2: R) -> Self {
        Self {
            w,
            x: v0,
            y: v1,
            z: v2,
        }
    }

    pub fn wxyz(&self) -> (R, R, R, R) {
        (self.w, self.x, self.y, self.z)
    }

    pub fn identity() -> Self {
        Self::new(1.into(), 0.into(), 0.into(), 0.into())
    }

    fn from_axis_angle(axis: Vec3, angle: R) -> Self {
        let w = cos(angle / 2.into());
        let c = sin(angle / 2.into());

        Self::new(w, c * axis.x, c * axis.y, c * axis.z)
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

    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag == 1.into() {
            return *self;
        }

        let mag = mag.sqrt();
        let w = self.w / mag;
        let x = self.x / mag;
        let y = self.y / mag;
        let z = self.z / mag;

        Self::new(w, x, y, z)
    }

    pub fn to_matrix(&self) -> [[R; 4]; 4] {
        let zero = R::zero();
        let one = R::one();
        let two: R = 2.into();

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

    pub fn rotate_vec3(&self, v: Vec3) -> Vec3 {
        // https://gamedev.stackexchange.com/a/50545

        let u = Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        };
        let s = self.w;

        let two = FixedNumber::from_i32(2);

        let v2 = u * two * u.dot(v) + v * (s.sqrd() - u.dot(u)) + u.cross(v) * two * s;

        v2
    }

    // Multiply two Quaternions. Not commutative, meaning q1 * q2 != q2 * q1.
    fn multiply(&self, other: Self) -> Self {
        let w = self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z;
        let x = self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y;
        let y = self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x;
        let z = self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w;

        // Check if we need to renormalize it.
        let m = Self::new(w, x, y, z);

        m
    }

    fn magnitude(&self) -> FixedNumber {
        self.w.sqrd() + self.x.sqrd() + self.y.sqrd() + self.z.sqrd()
    }

    fn should_normalize(&self) -> bool {
        let tolerance = FixedNumber::decimal_resolution_value() * FixedNumber::from_i32(10); // Tolerance is how much rounding errors we can tolerate
        let norm = self.magnitude();

        if FixedNumber::one() - tolerance < norm && FixedNumber::one() + tolerance > norm {
            return false;
        }

        true
    }
}

impl std::ops::Mul for Quaternion {
    type Output = Self;

    fn mul(self, rhs: Self) -> Quaternion {
        self.multiply(rhs)
    }
}

impl std::ops::MulAssign for Quaternion {
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

    #[test]
    fn Quaternion_to_matrix() {
        assert_eq!(true, false);
    }

    #[test]
    fn Quaternion_normalize() {
        let q = Quaternion::from_x_rotation(R::fraction(3.into()));

        println!("Q: {:?}", q);

        let expected = {
            println!("MAG: {:?}", q.magnitude());
            let mag = q.magnitude().sqrt();
            let w = q.w / mag;
            let x = q.x / mag;
            let y = q.y / mag;
            let z = q.z / mag;

            Quaternion::new(w, x, y, z)
        };

        assert_eq!(expected, q.normalize());
    }

    #[test]
    fn Quaternion_magnitude() {
        let q = Quaternion::from_x_rotation(R::fraction(3.into()));

        let expected = q.w.sqrd() + q.x.sqrd() + q.y.sqrd() + q.z.sqrd();
        let actual = q.magnitude();

        assert_eq!(expected, actual);

        let q = Quaternion::from_z_rotation(R::fraction(3.into()));

        let expected = q.w.sqrd() + q.x.sqrd() + q.y.sqrd() + q.z.sqrd();
        let actual = q.magnitude();

        assert_eq!(expected, actual);
    }

    #[test]
    fn Quaternion_mul_assign() {
        let mut q1 = Quaternion::from_x_rotation(R::fraction(3.into()));
        let other = Quaternion::from_y_rotation(4.into());

        let expected = q1.multiply(other);
        q1 *= other;

        assert_eq!(expected, q1);

        let mut q1 = Quaternion::from_z_rotation(R::fraction(3.into()));
        let other = Quaternion::from_y_rotation(4.into());

        let expected = q1.multiply(other);
        q1 *= other;

        assert_eq!(expected, q1);
    }

    #[test]
    fn Quaternion_mul() {
        let q1 = Quaternion::from_x_rotation(R::fraction(3.into()));
        let other = Quaternion::from_y_rotation(4.into());

        let expected = q1.multiply(other);
        let actual = q1 * other;

        assert_eq!(expected, actual);

        let q1 = Quaternion::from_z_rotation(R::fraction(3.into()));
        let other = Quaternion::from_x_rotation(4.into());

        let expected = q1.multiply(other);
        let actual = q1 * other;

        assert_eq!(expected, actual);
    }

    #[test]
    fn Quaternion_multiply() {
        let q1 = Quaternion::from_x_rotation(R::fraction(3.into()));
        let other = Quaternion::from_y_rotation(4.into());

        let w = q1.w * other.w - q1.x * other.x - q1.y * other.y - q1.z * other.z;
        let x = q1.w * other.x + q1.x * other.w + q1.y * other.z - q1.z * other.y;
        let y = q1.w * other.y - q1.x * other.z + q1.y * other.w + q1.z * other.x;
        let z = q1.w * other.z + q1.x * other.y - q1.y * other.x + q1.z * other.w;

        let expected = Quaternion::new(w, x, y, z);
        let actual = q1.multiply(other);
        assert_eq!(expected, actual);

        let q1 = Quaternion::from_z_rotation(R::fraction(3.into()));
        let other = Quaternion::from_x_rotation(44.into());

        let w = q1.w * other.w - q1.x * other.x - q1.y * other.y - q1.z * other.z;
        let x = q1.w * other.x + q1.x * other.w + q1.y * other.z - q1.z * other.y;
        let y = q1.w * other.y - q1.x * other.z + q1.y * other.w + q1.z * other.x;
        let z = q1.w * other.z + q1.x * other.y - q1.y * other.x + q1.z * other.w;

        let expected = Quaternion::new(w, x, y, z);
        let actual = q1.multiply(other);
        assert_eq!(expected, actual);
    }

    #[test]
    fn Quaternion_from_z_rotation() {
        let angle = R::fraction(7.into());

        let expected = Quaternion::from_axis_angle((0, 0, 1).into(), angle);
        let actual = Quaternion::from_z_rotation(angle);
        assert_eq!(expected, actual);

        let angle = R::fraction(37.into());

        let expected = Quaternion::from_axis_angle((0, 0, 1).into(), angle);
        let actual = Quaternion::from_z_rotation(angle);
        assert_eq!(expected, actual);

        let angle: R = (-3).into();

        let expected = Quaternion::from_axis_angle((0, 0, 1).into(), angle);
        let actual = Quaternion::from_z_rotation(angle);
        assert_eq!(expected, actual);

        let angle: R = (3).into();

        let expected = Quaternion::from_axis_angle((0, 0, 1).into(), angle);
        let actual = Quaternion::from_z_rotation(angle);
        assert_eq!(expected, actual);
    }
    #[test]
    fn Quaternion_from_y_rotation() {
        let angle = R::fraction(7.into());

        let expected = Quaternion::from_axis_angle((0, 1, 0).into(), angle);
        let actual = Quaternion::from_y_rotation(angle);
        assert_eq!(expected, actual);

        let angle = R::fraction(37.into());

        let expected = Quaternion::from_axis_angle((0, 1, 0).into(), angle);
        let actual = Quaternion::from_y_rotation(angle);
        assert_eq!(expected, actual);

        let angle: R = (-3).into();

        let expected = Quaternion::from_axis_angle((0, 1, 0).into(), angle);
        let actual = Quaternion::from_y_rotation(angle);
        assert_eq!(expected, actual);

        let angle: R = (3).into();

        let expected = Quaternion::from_axis_angle((0, 1, 0).into(), angle);
        let actual = Quaternion::from_y_rotation(angle);
        assert_eq!(expected, actual);
    }
    #[test]
    fn Quaternion_from_x_rotation() {
        let angle = R::fraction(7.into());

        let expected = Quaternion::from_axis_angle((1, 0, 0).into(), angle);
        let actual = Quaternion::from_x_rotation(angle);
        assert_eq!(expected, actual);

        let angle = R::fraction(37.into());

        let expected = Quaternion::from_axis_angle((1, 0, 0).into(), angle);
        let actual = Quaternion::from_x_rotation(angle);
        assert_eq!(expected, actual);

        let angle: R = (-3).into();

        let expected = Quaternion::from_axis_angle((1, 0, 0).into(), angle);
        let actual = Quaternion::from_x_rotation(angle);
        assert_eq!(expected, actual);

        let angle: R = (3).into();

        let expected = Quaternion::from_axis_angle((1, 0, 0).into(), angle);
        let actual = Quaternion::from_x_rotation(angle);
        assert_eq!(expected, actual);
    }

    #[test]
    fn Quaternion_from_axis_angle() {
        let axis = Vec3 {
            x: 0.into(),
            y: 2.into(),
            z: 3.into(),
        };
        let angle = R::fraction(7.into());

        let w = cos(angle / 2.into());
        let c = sin(angle / 2.into());

        let expected = Quaternion::new(w, c * axis.x, c * axis.y, c * axis.z);
        let actual = Quaternion::from_axis_angle(axis, angle);

        assert_eq!(expected, actual);

        let axis = Vec3 {
            x: 4.into(),
            y: 99.into(),
            z: 32.into(),
        };
        let angle = R::fraction(133.into());

        let w = cos(angle / 2.into());
        let c = sin(angle / 2.into());

        let expected = Quaternion::new(w, c * axis.x, c * axis.y, c * axis.z);
        let actual = Quaternion::from_axis_angle(axis, angle);

        assert_eq!(expected, actual);
    }

    #[test]
    fn Quaternion_new_sets_as_expected() {
        let w: R = 3.into();
        let v0: R = 4.into();
        let v1: R = 5.into();
        let v2: R = 6.into();

        let q = Quaternion::new(w, v0, v1, v2);

        assert_eq!(w, q.w);
        assert_eq!(v0, q.x);
        assert_eq!(v1, q.y);
        assert_eq!(v2, q.z);
    }

    #[test]
    fn Quaternion_identity_sets_as_expected() {
        let w: R = 1.into();
        let v0: R = 0.into();
        let v1: R = 0.into();
        let v2: R = 0.into();

        let q = Quaternion::identity();

        assert_eq!(w, q.w);
        assert_eq!(v0, q.x);
        assert_eq!(v1, q.y);
        assert_eq!(v2, q.z);

        let q1 = Quaternion::new(w, v0, v1, v2);

        assert_eq!(q1, q);
    }
}

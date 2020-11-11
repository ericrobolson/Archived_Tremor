use super::*;

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

    pub fn one() -> Self {
        Self {
            x: 1.into(),
            y: 1.into(),
            z: 1.into(),
        }
    }

    pub fn dot(&self, other: Self) -> FixedNumber {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: Self) -> Self {
        Self {
            x: (self.y * other.z - self.z * other.y),
            y: (self.z * other.x - self.x * other.z),
            z: (self.x * other.y - self.y * other.x),
        }
    }

    pub fn normalize(&self) -> Self {
        let len = self.len();

        *self / len
    }

    pub fn len_squared(&self) -> FixedNumber {
        self.x.sqrd() + self.y.sqrd() + self.z.sqrd()
    }

    pub fn len(&self) -> FixedNumber {
        self.len_squared().sqrt()
    }

    pub fn unit_y() -> Self {
        Self {
            x: 0.into(),
            y: 1.into(),
            z: 0.into(),
        }
    }
}

impl Into<[f32; 3]> for Vec3 {
    fn into(self) -> [f32; 3] {
        [self.x.into(), self.y.into(), self.z.into()]
    }
}

impl Into<Vec3> for FixedNumber {
    fn into(self) -> Vec3 {
        Vec3 {
            x: self,
            y: self,
            z: self,
        }
    }
}

impl Into<Vec3> for (FixedNumber, FixedNumber, FixedNumber) {
    fn into(self) -> Vec3 {
        Vec3 {
            x: self.0,
            y: self.1,
            z: self.2,
        }
    }
}

impl Into<Vec3> for (i32, i32, i32) {
    fn into(self) -> Vec3 {
        Vec3 {
            x: self.0.into(),
            y: self.1.into(),
            z: self.2.into(),
        }
    }
}

impl std::ops::Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> <Self as std::ops::Add<Self>>::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl std::ops::AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl std::ops::SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> <Self as std::ops::Sub<Self>>::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl std::ops::Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> <Self as std::ops::Neg>::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

// FixedNumber ops
impl std::ops::Mul<FixedNumber> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: FixedNumber) -> Vec3 {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl std::ops::Mul for Vec3 {
    type Output = Self;

    fn mul(self, rhs: Self) -> <Self as std::ops::Mul<Self>>::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl std::ops::MulAssign for Vec3 {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl std::ops::Div<FixedNumber> for Vec3 {
    type Output = Self;

    fn div(self, rhs: FixedNumber) -> Vec3 {
        if rhs == 0.into() {
            panic!("Divide Vec3 by zero!");
        }

        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn Vec3_dot_returns_expected() {
        let vec1: Vec3 = (1, 2, 3).into();
        let vec2: Vec3 = (5, 6, 7).into();

        let expected: FixedNumber = (1 * 5 + 2 * 6 + 3 * 7).into();
        let actual = vec1.dot(vec2);
        assert_eq!(expected, actual);

        let actual = vec2.dot(vec1);
        assert_eq!(expected, actual);

        let vec1: Vec3 = (6, -7, 0).into();
        let vec2: Vec3 = (5, 6, 7).into();

        let expected: FixedNumber = (6 * 5 + -7 * 6 + 0 * 7).into();
        let actual = vec1.dot(vec2);
        assert_eq!(expected, actual);

        let actual = vec2.dot(vec1);
        assert_eq!(expected, actual);
    }

    #[test]
    fn Vec3_normalize_returns_expected() {
        let a: FixedNumber = 3.into();
        let b: FixedNumber = 9.into();
        let c: FixedNumber = 11.into();

        let vec1: Vec3 = (a, b, c).into();

        let len = (a.sqrd() + b.sqrd() + c.sqrd()).sqrt();

        let expected: Vec3 = (a / len, b / len, c / len).into();
        let actual = vec1.normalize();
        assert_eq!(expected, actual);

        let a: FixedNumber = (-34).into();
        let b: FixedNumber = 32.into();
        let c: FixedNumber = 0.into();

        let vec1: Vec3 = (a, b, c).into();

        let len = (a.sqrd() + b.sqrd() + c.sqrd()).sqrt();

        let expected: Vec3 = (a / len, b / len, c / len).into();
        let actual = vec1.normalize();
        assert_eq!(expected, actual);
    }
}

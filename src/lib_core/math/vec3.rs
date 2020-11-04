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

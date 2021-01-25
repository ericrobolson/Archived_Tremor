use super::*;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Vec3<R>
where
    R: Number,
{
    pub x: R,
    pub y: R,
    pub z: R,
}

impl<R> Vec3<R>
where
    R: Number,
{
    pub fn new() -> Self {
        (0, 0, 0).into()
    }

    pub fn one() -> Self {
        (1, 1, 1).into()
    }

    pub fn dot(&self, other: Self) -> R {
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

    pub fn len_squared(&self) -> R {
        self.x.psqrd() + self.y.psqrd() + self.z.psqrd()
    }

    pub fn len(&self) -> R {
        self.len_squared().psqrt()
    }

    pub fn unit_y() -> Self {
        (0, 1, 0).into()
    }

    // Taking the componentwise maximum
    pub fn componentwise_max(&self, other: Self) -> Self {
        Self {
            x: R::max(self.x, other.x),
            y: R::max(self.y, other.y),
            z: R::max(self.z, other.z),
        }
    }

    // Taking the componentwise minimum
    pub fn componentwise_min(&self, other: Self) -> Self {
        Self {
            x: R::min(self.x, other.x),
            y: R::min(self.y, other.y),
            z: R::min(self.z, other.z),
        }
    }
}

impl<R> Into<[f32; 3]> for Vec3<R>
where
    R: Number,
{
    fn into(self) -> [f32; 3] {
        [self.x.to_f32(), self.y.to_f32(), self.z.to_f32()]
    }
}

impl<R> Into<Vec3<R>> for (i32, i32, i32)
where
    R: Number,
{
    fn into(self) -> Vec3<R> {
        Vec3 {
            x: R::from(self.0),
            y: R::from(self.1),
            z: R::from(self.2),
        }
    }
}

impl<R> Into<Vec3<R>> for (R, R, R)
where
    R: Number,
{
    fn into(self) -> Vec3<R> {
        Vec3 {
            x: self.0,
            y: self.1,
            z: self.2,
        }
    }
}
impl<R> std::ops::Add for Vec3<R>
where
    R: Number,
{
    type Output = Self;

    fn add(self, rhs: Self) -> <Self as std::ops::Add<Self>>::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<R> std::ops::AddAssign for Vec3<R>
where
    R: Number,
{
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl<R> std::ops::SubAssign for Vec3<R>
where
    R: Number,
{
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl<R> std::ops::Sub for Vec3<R>
where
    R: Number,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> <Self as std::ops::Sub<Self>>::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl<R> std::ops::Neg for Vec3<R>
where
    R: Number,
{
    type Output = Self;

    fn neg(self) -> <Self as std::ops::Neg>::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl<R> std::ops::Mul<R> for Vec3<R>
where
    R: Number,
{
    type Output = Self;

    fn mul(self, rhs: R) -> Vec3<R> {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl<R> std::ops::Mul for Vec3<R>
where
    R: Number,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> <Self as std::ops::Mul<Self>>::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl<R> std::ops::MulAssign for Vec3<R>
where
    R: Number,
{
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl<R> std::ops::Div<R> for Vec3<R>
where
    R: Number,
{
    type Output = Self;

    fn div(self, rhs: R) -> Vec3<R> {
        if rhs == R::zero() {
            panic!("Divide Vec3 by zero!");
        }

        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

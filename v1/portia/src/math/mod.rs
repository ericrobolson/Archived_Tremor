pub mod quartenion;
pub mod vec3;

mod number_impls;

// TODO: make this generic, but pull over code from raytracer. I like those ops better.
pub struct Vec3 {}
// TODO: make this generic, but pull over code from raytracer
pub struct Quartenion {}
// Point in space.
pub struct Point3 {}
pub trait PortiaOps {
    fn psin(&self) -> Self;
    fn pcos(&self) -> Self;
    fn psqrt(&self) -> Self;
    fn fraction(numerator: i32, denominator: i32) -> Self;
    fn to_f32(&self) -> f32;

    fn from(i: i32) -> Self;
}

pub trait Number:
    Copy
    + Clone
    + PartialEq
    + PortiaOps
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::Mul<Output = Self>
    + std::ops::Div<Output = Self>
    + std::ops::Neg<Output = Self>
    + std::ops::AddAssign
    + std::ops::SubAssign
    + std::ops::MulAssign
    + std::ops::DivAssign
    + std::cmp::PartialOrd
where
    Self: std::marker::Sized,
{
    fn zero() -> Self;
    fn one() -> Self;
    fn pabs(&self) -> Self;
    fn psqrd(&self) -> Self;
    fn min(a: Self, b: Self) -> Self;
    fn max(a: Self, b: Self) -> Self;
}

impl<R> Number for R
where
    R: Copy
        + Clone
        + PartialEq
        + PortiaOps
        + std::ops::Add<Output = Self>
        + std::ops::Sub<Output = Self>
        + std::ops::Mul<Output = Self>
        + std::ops::Div<Output = Self>
        + std::ops::Neg<Output = Self>
        + std::ops::AddAssign
        + std::ops::SubAssign
        + std::ops::MulAssign
        + std::ops::DivAssign
        + std::cmp::PartialOrd,
{
    fn zero() -> Self {
        Self::from(0)
    }

    fn one() -> Self {
        Self::from(1)
    }

    fn pabs(&self) -> Self {
        let v = *self;
        if v < Self::zero() {
            return -v;
        }

        v
    }

    fn psqrd(&self) -> Self {
        *self * *self
    }

    fn min(a: Self, b: Self) -> Self {
        if a < b {
            a
        } else {
            b
        }
    }
    fn max(a: Self, b: Self) -> Self {
        if a > b {
            a
        } else {
            b
        }
    }
}

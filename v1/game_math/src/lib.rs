#[macro_use]
mod macros;

pub mod f32;
pub mod fix;
pub mod mat4;
pub mod quaternion;
pub mod vec2;
pub mod vec3;
pub mod vec4;

/// A list of all features a number must provide.
pub trait Number:
    Sized
    + std::ops::Add<Output = Self>
    + std::ops::AddAssign
    + std::ops::Sub<Output = Self>
    + std::ops::SubAssign
    + std::ops::Mul<Output = Self>
    + std::ops::MulAssign
    + std::ops::Div<Output = Self>
    + std::ops::DivAssign
    + std::ops::Neg<Output = Self>
    //+ std::cmp::Ord
    + std::cmp::PartialOrd
    + std::cmp::PartialEq
    //+ std::cmp::Eq
    + Copy
    + Clone
    + std::fmt::Debug
{
    /// Creates a Number from an i32.
    fn i32(real_number: i32) -> Self;

    /// Returns the absolute value of a number.
    /// ## Example
    /// ```
    /// let n = r(23);
    /// let m = r(-23);
    /// assert_eq!(m.abs(), n.abs());
    ///
    /// let n = nd(23);
    /// let m = nd(-23);
    /// assert_eq!(m.abs(), n.abs());
    /// ```
    fn abs(&self) -> Self;
    /// Returns the square root of a number.
    /// ## Example
    /// ```
    /// let n = d(4);
    /// assert_eq!(d(2), n.sqrt());
    ///
    /// let n = nd(4);
    /// assert_eq!(nd(2), n.sqrt());
    /// ```
    fn sqrt(&self) -> Self;

    /// Returns the squared value of a Number.
    fn sqrd(&self) -> Self;
    /// Returns the sine of a number.
    fn sin(&self) -> Self;
    /// Returns the cosine of a number.
    fn cos(&self) -> Self;
    /// Returns the min of a number.
    fn min(&self, other: Self) -> Self;
    /// Returns the max of a number.
    fn max(&self, other: Self) -> Self;
}

/// Converts a data structure to a raw type. Useful for uploading to the GPU.
pub trait RawConverter {
    type RawType;

    /// Converts the higher level type to a low level type. E.g. Vec3.to_raw() = [f32;3].
    fn to_raw(&self) -> Self::RawType;

    /// Converts the low level type to a higher level type. E.g Vec3::from_raw([f32;3]);
    fn from_raw(raw: Self::RawType) -> Self;
}

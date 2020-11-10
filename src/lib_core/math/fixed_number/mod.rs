extern crate fixed;
extern crate fixed_sqrt;

use fixed::traits::Fixed;
use fixed::types::*;
use fixed_sqrt::FixedSqrt;

use fixed::types::I20F12;
pub type FIX = I20F12;

use std::str::FromStr;

mod lookup_generation;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct FixedNumber {
    value: FIX,
}

impl FixedNumber {
    fn from_fix(value: FIX) -> Self {
        Self { value: value }
    }

    pub fn MAX() -> Self {
        429496.into()
    }

    pub fn PI() -> Self {
        Self { value: FIX::PI }
    }

    pub fn TWO_PI() -> Self {
        Self { value: FIX::PI } * Self::from_i32(2)
    }

    pub fn min(a: Self, b: Self) -> Self {
        if a.value <= b.value {
            return a;
        }

        b
    }

    pub fn one() -> Self {
        1.into()
    }

    pub fn zero() -> Self {
        0.into()
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        self.value.to_le_bytes()
    }

    pub fn from_bytes(bytes: [u8; 4]) -> Self {
        let value = FIX::from_le_bytes(bytes);

        Self { value }
    }

    pub fn fraction(num: FixedNumber) -> Self {
        if num == Self::zero() {
            panic!("Divide by zero!");
        }

        Self::one() / num
    }

    /// Squared function
    pub fn sqrd(&self) -> Self {
        let value = *self;

        value * value
    }

    /// Square root function
    pub fn sqrt(&self) -> Self {
        let v = self.value.sqrt();
        Self::from_fix(v)
    }

    pub fn remainder(&self, other: Self) -> Self {
        Self {
            value: self.value.rem_euclid(other.value),
        }
    }

    pub fn abs(&self) -> Self {
        let value = *self;

        if value < 0.into() {
            return value * -1;
        }

        value
    }

    pub fn max(a: Self, b: Self) -> Self {
        if a.value <= b.value {
            return b;
        }

        a
    }

    pub fn from_str(s: String) -> Self {
        Self {
            value: FIX::from_str(s.as_str()).unwrap(),
        }
    }

    pub fn from_i32(number: i32) -> Self {
        Self {
            value: FIX::from_num(number),
        }
    }
}

impl std::ops::Add for FixedNumber {
    type Output = Self;

    fn add(self, rhs: Self) -> <Self as std::ops::Add<Self>>::Output {
        Self {
            value: self.value + rhs.value,
        }
    }
}

impl std::ops::Neg for FixedNumber {
    type Output = Self;
    fn neg(self) -> <Self as std::ops::Neg>::Output {
        Self { value: -self.value }
    }
}

impl std::ops::AddAssign for FixedNumber {
    fn add_assign(&mut self, rhs: Self) {
        self.value += rhs.value;
    }
}

impl std::ops::Sub for FixedNumber {
    type Output = Self;

    fn sub(self, rhs: Self) -> <Self as std::ops::Sub<Self>>::Output {
        Self {
            value: self.value - rhs.value,
        }
    }
}

impl std::ops::SubAssign for FixedNumber {
    fn sub_assign(&mut self, rhs: Self) {
        self.value -= rhs.value;
    }
}

impl std::ops::Mul for FixedNumber {
    type Output = Self;
    fn mul(self, rhs: Self) -> <Self as std::ops::Mul<Self>>::Output {
        Self {
            value: self.value * rhs.value,
        }
    }
}
impl std::ops::Mul<i32> for FixedNumber {
    type Output = Self;

    fn mul(self, rhs: i32) -> FixedNumber {
        let rhs: FixedNumber = rhs.into();
        self * rhs
    }
}

impl std::ops::MulAssign for FixedNumber {
    fn mul_assign(&mut self, rhs: Self) {
        self.value *= rhs.value;
    }
}

impl std::ops::Div for FixedNumber {
    type Output = Self;
    fn div(self, rhs: Self) -> <Self as std::ops::Div<Self>>::Output {
        Self {
            value: self.value / rhs.value,
        }
    }
}

impl std::ops::DivAssign for FixedNumber {
    fn div_assign(&mut self, rhs: Self) {
        self.value /= rhs.value;
    }
}

impl Into<FixedNumber> for i32 {
    fn into(self) -> FixedNumber {
        FixedNumber::from_i32(self)
    }
}

impl Into<FixedNumber> for usize {
    fn into(self) -> FixedNumber {
        let i = self as i32;
        FixedNumber::from_i32(i)
    }
}

impl Into<f32> for FixedNumber {
    fn into(self) -> f32 {
        self.value.to_num::<f32>()
    }
}

impl Into<usize> for FixedNumber {
    fn into(self) -> usize {
        if self <= 0.into() {
            return 0;
        }

        let i = self.value.to_num::<i32>();

        return i as usize;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn FixedNumber_sqrt4_2() {
        let value: FixedNumber = 4.into();

        let expected: FixedNumber = 2.into();
        let actual: FixedNumber = value.sqrt();
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumber_sqrt9_3() {
        let value: FixedNumber = 9.into();

        let expected: FixedNumber = 3.into();
        let actual: FixedNumber = value.sqrt();
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumber_sqrd2_4() {
        let value: FixedNumber = 2.into();

        let expected: FixedNumber = 4.into();
        let actual: FixedNumber = value.sqrd();
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumber_sqrd9_81() {
        let value: FixedNumber = 9.into();

        let expected: FixedNumber = 81.into();
        let actual: FixedNumber = value.sqrd();
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumber_divide() {
        let expected = FixedNumber::from_i32(0);
        let v1 = FixedNumber::from_i32(0);
        let v2 = FixedNumber::from_i32(2000);

        assert_eq!(expected, v1 / v2);

        let expected = FIX::from_num(2) / FIX::from_num(3);

        let v1 = FixedNumber::from_i32(2);
        let v2 = FixedNumber::from_i32(3);

        assert_eq!(expected, (v1 / v2).value);
    }

    #[test]
    fn FixedNumber_divide_assign() {
        let expected = FixedNumber::from_i32(0);
        let mut v1 = FixedNumber::from_i32(0);
        let v2 = FixedNumber::from_i32(2000);

        v1 /= v2;

        assert_eq!(expected, v1);

        let expected = FIX::from_num(2) / FIX::from_num(3);

        let mut v1 = FixedNumber::from_i32(2);
        let v2 = FixedNumber::from_i32(3);

        v1 /= v2;

        assert_eq!(expected, v1.value);
    }

    #[test]
    fn FixedNumber_subtract() {
        let expected = FixedNumber::from_i32(-2000);

        let v1 = FixedNumber::from_i32(0);
        let v2 = FixedNumber::from_i32(2000);

        assert_eq!(expected, v1 - v2);

        let expected = FixedNumber::from_i32(2000);

        let v1 = FixedNumber::from_i32(0);
        let v2 = FixedNumber::from_i32(-2000);

        assert_eq!(expected, v1 - v2);
    }

    #[test]
    fn FixedNumber_subtract_assign() {
        let expected = FixedNumber::from_i32(-2000);

        let mut v1 = FixedNumber::from_i32(0);
        let v2 = FixedNumber::from_i32(2000);

        v1 -= v2;

        assert_eq!(expected, v1);

        let expected = FixedNumber::from_i32(2000);

        let mut v1 = FixedNumber::from_i32(0);
        let v2 = FixedNumber::from_i32(-2000);

        v1 -= v2;

        assert_eq!(expected, v1);
    }

    #[test]
    fn FixedNumber_add() {
        let expected = FixedNumber::from_i32(2000);

        let v1 = FixedNumber::from_i32(0);
        let v2 = FixedNumber::from_i32(2000);

        assert_eq!(expected, v1 + v2);

        let expected = FixedNumber::from_i32(-1);

        let v1 = FixedNumber::from_i32(-2001);
        let v2 = FixedNumber::from_i32(2000);

        assert_eq!(expected, v1 + v2);
    }

    #[test]
    fn FixedNumber_add_assign() {
        let expected = FixedNumber::from_i32(2000);

        let mut v1 = FixedNumber::from_i32(0);
        let v2 = FixedNumber::from_i32(2000);

        v1 += v2;

        assert_eq!(expected, v1);

        let expected = FixedNumber::from_i32(-222);

        let mut v1 = FixedNumber::from_i32(-2222);
        let v2 = FixedNumber::from_i32(2000);

        v1 += v2;

        assert_eq!(expected, v1);
    }

    #[test]
    fn FixedNumber_from_i32_0() {
        let num = 0;
        let value = FIX::from_num(num);
        let fixed_number = FixedNumber::from_i32(num);

        assert_eq!(value, fixed_number.value);
    }

    #[test]
    fn FixedNumber_from_i32_1001() {
        let num = 1001;
        let value = FIX::from_num(num);
        let fixed_number = FixedNumber::from_i32(num);

        assert_eq!(value, fixed_number.value);
    }

    #[test]
    fn FixedNumber_from_i32_n2030() {
        let num = -2030;
        let value = FIX::from_num(num);
        let fixed_number = FixedNumber::from_i32(num);

        assert_eq!(value, fixed_number.value);
    }
}

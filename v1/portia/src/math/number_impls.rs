use super::*;

impl PortiaOps for f32 {
    fn pcos(&self) -> Self {
        self.cos()
    }

    fn psin(&self) -> Self {
        self.sin()
    }

    fn psqrt(&self) -> Self {
        self.sqrt()
    }

    fn to_f32(&self) -> f32 {
        *self
    }

    fn from(i: i32) -> Self {
        i as Self
    }

    fn fraction(numerator: i32, denominator: i32) -> Self {
        let n = numerator as Self;
        let d = denominator as Self;

        return n / d;
    }
}

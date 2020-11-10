use super::*;

use lazy_static::*;
lazy_static! {
    pub static ref TRIG_LUT: FixedNumberLut = { from_file() };
}

pub struct FixedNumberLut {
    min_val: FixedNumber,
    sines: Vec<FixedNumber>,
}

impl FixedNumberLut {
    fn new() -> Self {
        let (min_val, sines) = generate_lut();

        Self { min_val, sines }
    }

    fn from_bytes(bytes: Vec<u8>) -> Self {
        from_bytes(bytes)
    }

    pub fn sine(&self, theta: FixedNumber) -> FixedNumber {
        // Map the number to a 0..2PI range
        let theta = wrap_theta(theta);

        // Get index
        let index = index_into_lut(theta, &self);

        // Return the value
        self.sines[index]
    }

    pub fn cosine(&self, theta: FixedNumber) -> FixedNumber {
        // Adjust theta by half PI, as that's what cosine starts at
        let theta = theta + FixedNumber::PI() / 2.into();

        self.sine(theta)
    }
}

fn from_file() -> FixedNumberLut {
    let bytes = include_bytes!("sine.lookup");

    from_bytes((&bytes).to_vec())
}

fn from_bytes(bytes: Vec<u8>) -> FixedNumberLut {
    let mut bytes = bytes;

    // Read in other bytes. A little hacky, but basically taking every 4 elements and deserializing.
    let mut vals = {
        let capacity = bytes.len() / 4;
        let mut sines = Vec::with_capacity(capacity);
        for i in 0..capacity {
            let j = i * 4;
            let f = FixedNumber::from_bytes([bytes[j], bytes[j + 1], bytes[j + 2], bytes[j + 3]]);
            sines.push(f);
        }

        sines
    };

    let min_val = vals.remove(vals.len() - 1); // Read from end
    let sines = vals;

    FixedNumberLut { min_val, sines }
}

// Map theta to a 0..2PI range
fn wrap_theta(theta: FixedNumber) -> FixedNumber {
    if theta > 0.into() && theta < FixedNumber::TWO_PI() {
        return theta;
    }

    // Convert to positive if negative
    let theta = {
        if theta < 0.into() {
            let r = theta.remainder(FixedNumber::TWO_PI()) + FixedNumber::TWO_PI();
            r
        } else {
            theta
        }
    };

    // Get the remainder of the value (equivalent of modulo)
    if theta >= FixedNumber::TWO_PI() {
        return theta.remainder(FixedNumber::TWO_PI());
    }

    theta
}

fn index_into_lut(theta: FixedNumber, lut: &FixedNumberLut) -> usize {
    // Map the number to a 0..2PI range
    let theta = wrap_theta(theta);

    // Get index
    let index: usize = (theta / lut.min_val).into();
    let index: usize = index % lut.sines.len();

    index
}

#[cfg(not(feature = "compile-lookups"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn FixedNumberLookup_lut_min_val_equals_generated_min_val() {
        let lut = FixedNumberLut::new();
        let byte_lut = from_file();

        assert_eq!(byte_lut.min_val, lut.min_val);
    }

    #[test]
    fn FixedNumberLookup_lut_sines_equals_generated_sines() {
        // If this fails, rebuild the sines using the 'compile-lookups' feature
        let lut = FixedNumberLut::new();
        let byte_lut = from_file();

        for i in 0..byte_lut.sines.len() {
            println!("i: {:?}", i);
            assert_eq!(byte_lut.sines[i], lut.sines[i]);
        }
    }

    #[test]
    fn FixedNumberLookup_sine_zero() {
        let val: FixedNumber = FixedNumber::zero();
        let actual = TRIG_LUT.sine(val);
        let expected: FixedNumber = {
            let f: f32 = val.into();
            let sine = f32::sin(f);
            let f = FixedNumber::from_fix(FIX::from_num(sine));

            f
        };
        assert_eq!(expected, actual);
    }
    #[test]
    fn FixedNumberLookup_sine_one() {
        let lut = FixedNumberLut::new();

        let val: FixedNumber = FixedNumber::one();
        let actual = lut.sine(val);
        let expected: FixedNumber = {
            let f: f32 = val.into();
            let sine = f32::sin(f);
            let f = FixedNumber::from_fix(FIX::from_num(sine));

            f
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_sine_pi_tests() {
        let lut = FixedNumberLut::new();

        let val: FixedNumber = FixedNumber::PI();
        let actual = lut.sine(val);
        let expected: FixedNumber = {
            let f: f32 = val.into();
            let sine = f32::sin(f);
            let f = FixedNumber::from_fix(FIX::from_num(sine));

            f
        };
        assert_eq!(expected, actual);

        let val: FixedNumber = FixedNumber::PI() / 2.into();
        let actual = lut.sine(val);
        let expected: FixedNumber = {
            let f: f32 = val.into();
            let sine = f32::sin(f);
            let f = FixedNumber::from_fix(FIX::from_num(sine));

            f
        };
        assert_eq!(expected, actual);

        let val: FixedNumber = FixedNumber::PI() / 3.into();
        let actual = lut.sine(val);
        let expected: FixedNumber = {
            let f: f32 = val.into();
            let sine = f32::sin(f);
            let f = FixedNumber::from_fix(FIX::from_num(sine));

            f
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_cosine_zero() {
        let val: FixedNumber = FixedNumber::zero();
        let actual = TRIG_LUT.cosine(val);
        let expected: FixedNumber = {
            let theta = (FixedNumber::PI() / 2.into()) + val;
            let f: f32 = val.into();
            let sine = f32::cos(f);
            let f = FixedNumber::from_fix(FIX::from_num(sine));

            f
        };
        assert_eq!(expected, actual);
    }
    #[test]
    fn FixedNumberLookup_cosine_one() {
        let lut = FixedNumberLut::new();

        let val: FixedNumber = FixedNumber::one();
        let actual = lut.cosine(val);
        let expected: FixedNumber = {
            let f: f32 = val.into();
            let sine = f32::cos(f);
            let f = FixedNumber::from_fix(FIX::from_num(sine));

            f
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_cosine_pi_tests() {
        let lut = FixedNumberLut::new();

        let val: FixedNumber = FixedNumber::PI();
        let actual = lut.cosine(val);
        let expected: FixedNumber = {
            let f: f32 = val.into();
            let sine = f32::cos(f);
            let f = FixedNumber::from_fix(FIX::from_num(sine));

            f
        };
        assert_eq!(expected, actual);

        let val: FixedNumber = FixedNumber::PI() / 2.into();
        let actual = lut.cosine(val);
        let expected: FixedNumber = {
            let f: f32 = val.into();
            let sine = f32::cos(f);
            let f = FixedNumber::from_fix(FIX::from_num(sine));

            f
        };
        assert_eq!(expected, actual);

        let val: FixedNumber = FixedNumber::PI() / 3.into();
        let actual = lut.cosine(val);
        let expected: FixedNumber = {
            let f: f32 = val.into();
            let sine = f32::cos(f);
            let f = FixedNumber::from_fix(FIX::from_num(sine));

            f
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_index_into_lut() {
        let lut = FixedNumberLut::new();
        let lut_len = lut.sines.len();

        let theta: FixedNumber = 0.into();
        let expected = 0;
        let actual = index_into_lut(theta, &lut);
        assert_eq!(expected, actual);

        let theta: FixedNumber = lut.min_val;
        let expected = 1;
        let actual = index_into_lut(theta, &lut);
        assert_eq!(expected, actual);

        let theta: FixedNumber = lut.min_val * 2;
        let expected = 2;
        let actual = index_into_lut(theta, &lut);
        assert_eq!(expected, actual);

        for i in 0..lut_len {
            let theta: FixedNumber = lut.min_val * (i as i32);
            let expected = i;
            let actual = index_into_lut(theta, &lut);
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_0case() {
        // 0 case
        let theta: FixedNumber = 0.into();
        let expected: FixedNumber = 0.into();
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_halfpi_case() {
        // 1/2 pi case
        let theta: FixedNumber = FixedNumber::PI() / 2.into();
        let expected: FixedNumber = theta;
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_neg_halfpi_case() {
        let theta: FixedNumber = -(FixedNumber::PI() / 2.into());
        let expected: FixedNumber = FixedNumber::TWO_PI() + theta;
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_pi_case() {
        // pi case
        let theta: FixedNumber = FixedNumber::PI();
        let expected: FixedNumber = theta;
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_neg_pi_case() {
        let theta: FixedNumber = -(FixedNumber::PI());
        let expected: FixedNumber = FixedNumber::TWO_PI() + theta;
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_one_and_half_pi_case() {
        // 1.5 pi case
        let theta: FixedNumber = FixedNumber::PI() + FixedNumber::PI() / 2.into();
        let expected: FixedNumber = theta;
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_neg_one_and_half_pi_case() {
        let theta: FixedNumber = -(FixedNumber::PI() + FixedNumber::PI() / 2.into());
        let expected: FixedNumber = FixedNumber::TWO_PI() + theta;
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_two_pi_case() {
        // 2 pi case
        let theta: FixedNumber = FixedNumber::TWO_PI();
        let expected: FixedNumber = 0.into();
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_neg_two_pi_case() {
        let theta: FixedNumber = -(FixedNumber::TWO_PI());
        let expected: FixedNumber = 0.into();
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_two_and_half_pi_case() {
        // 2 pi + 1/ PI case = 1/2 pi
        let theta: FixedNumber = FixedNumber::TWO_PI() + FixedNumber::PI() / 2.into();
        let expected: FixedNumber = FixedNumber::PI() / 2.into();
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_neg_two_and_half_pi_case() {
        let theta: FixedNumber = -(FixedNumber::TWO_PI() + FixedNumber::PI() / 2.into());
        let expected: FixedNumber = FixedNumber::TWO_PI() - FixedNumber::PI() / 2.into();
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_three_pi_case() {
        let theta: FixedNumber = FixedNumber::TWO_PI() + FixedNumber::PI();
        let expected: FixedNumber = FixedNumber::PI();
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_neg_three_pi_case() {
        let theta: FixedNumber = -(FixedNumber::TWO_PI() + FixedNumber::PI());
        let expected: FixedNumber = FixedNumber::PI();
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_three_and_half_pi_case() {
        let theta: FixedNumber =
            FixedNumber::TWO_PI() + FixedNumber::PI() + FixedNumber::PI() / 2.into();
        let expected: FixedNumber = FixedNumber::PI() + FixedNumber::PI() / 2.into();
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_4_pi_case() {
        let theta: FixedNumber = FixedNumber::TWO_PI() + FixedNumber::TWO_PI();
        let expected: FixedNumber = FixedNumber::zero();
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_neg_4_pi_case() {
        let theta: FixedNumber = -(FixedNumber::TWO_PI() + FixedNumber::TWO_PI());
        let expected: FixedNumber = FixedNumber::zero();
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }
}

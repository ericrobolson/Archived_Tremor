use super::*;

use lazy_static::*;
lazy_static! {
    static ref TRIG_LUT: FixedNumberLut = {
        // TODO: change this to pull in precalculated bytes
        FixedNumberLut::new()
     };
}

fn decimal_resolution_value() -> FixedNumber {
    // Find the maximum iterations we can run
    let mut j = 1;
    while (FIX::from_num(1) / j) > FIX::from_num(0) {
        j += 1;
    }

    j -= 1; // Ensure we get the last value before it went to 0

    let i = FIX::from_num(1) / (j); // Get the smallest representable fixed number

    FixedNumber::from_fix(i)
}

pub struct FixedNumberLut {
    min_val: FixedNumber,
    sines: Vec<FixedNumber>,
}

impl FixedNumberLut {
    fn new() -> Self {
        let (min_val, sines) = Self::generate(); // TODO: instead import bytes from precalculated lut

        Self { min_val, sines }
    }

    fn generate() -> (FixedNumber, Vec<FixedNumber>) {
        let start = FixedNumber::zero();
        let increment = decimal_resolution_value();
        let end = FixedNumber::TWO_PI(); // TODO: may even be able to cut LUT in half by only doing 0..PI and then reversing the indexes after a certain point?

        let mut value = start;
        let mut sines = vec![];
        while value < end {
            let sin_lookup = f32::sin(value.into());
            let sin_lookup = FixedNumber::from_fix(FIX::from_num(sin_lookup));
            sines.push(sin_lookup);

            // Finally increment the value to calculate
            value += increment;
        }

        (increment, sines)
    }

    fn generate_bytes() -> Vec<u8> {
        let (min_size, nums) = Self::generate();

        // First number is the min size
        let mut min_size: Vec<u8> = min_size.to_bytes().iter().map(|d| *d).collect();

        // Rest of the numbers are the actual bytes
        let mut bytes: Vec<u8> = nums
            .iter()
            .map(|n| n.to_bytes())
            .collect::<Vec<[u8; 4]>>()
            .iter()
            .flat_map(|d| d.iter())
            .map(|d| *d)
            .collect();

        min_size.append(&mut bytes);

        min_size
    }

    fn from_bytes(bytes: Vec<u8>) -> FixedNumberLut {
        let mut bytes = bytes;
        let min_size = [
            bytes.pop().unwrap(),
            bytes.pop().unwrap(),
            bytes.pop().unwrap(),
            bytes.pop().unwrap(),
        ];
        let min_val = FixedNumber::from_bytes(min_size);

        // Read in other bytes. A little hacky, but basically taking every 4 elements and deserializing.
        let capacity = bytes.len() / 4;
        let mut sines = Vec::with_capacity(capacity);
        for i in 0..capacity {
            let i = i * 4;
            let f = FixedNumber::from_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
            sines.push(f);
        }

        Self { min_val, sines }
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

#[cfg(test)]
mod tests {
    use super::*;

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

use super::*;

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
    cosines: Vec<FixedNumber>,
}

impl FixedNumberLut {
    pub fn new() -> Self {
        let (min_val, sines, cosines) = Self::generate(); // TODO: instead import bytes from precalculated lut

        Self {
            min_val,
            sines,
            cosines,
        }
    }

    fn generate() -> (FixedNumber, Vec<FixedNumber>, Vec<FixedNumber>) {
        let start = FixedNumber::zero();
        let increment = decimal_resolution_value();
        let end = FixedNumber::TWO_PI(); // TODO: may even be able to cut LUT in half by only doing 0..PI and then reversing the indexes after a certain point?

        let mut value = start;
        let mut cosines = vec![];
        let mut sines = vec![];
        while value < end {
            let sin_lookup = f32::sin(value.into());
            let sin_lookup = FixedNumber::from_fix(FIX::from_num(sin_lookup));
            sines.push(sin_lookup);

            let cos_lookup = f32::cos(value.into()); // TODO: may be able to just offset the sin function
            let cos_lookup = FixedNumber::from_fix(FIX::from_num(cos_lookup));
            cosines.push(cos_lookup);

            // Finally increment the value to calculate
            value += increment;
        }

        (increment, sines, cosines)
    }

    pub fn sine(&self, theta: FixedNumber) -> FixedNumber {
        // Map the number to a 0..2PI range
        let theta = wrap_theta(theta);

        // Get index
        let index: usize = (theta / self.min_val).into();
        let index: usize = index % self.sines.len();

        // Return the value
        self.sines[index]
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
    let two_pi = FixedNumber::TWO_PI();
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
    fn FixedNumberLookup_Sin_Test() {
        let lut = FixedNumberLut::new();

        let val: FixedNumber = FixedNumber::zero();
        let actual = lut.sine(val);
        let expected: FixedNumber = {
            let f: f32 = val.into();
            let sine = f32::sin(f);
            let f = FixedNumber::from_fix(FIX::from_num(sine));

            f
        };
        assert_eq!(expected, actual);

        let val: FixedNumber = FixedNumber::one();
        let actual = lut.sine(val);
        let expected: FixedNumber = {
            let f: f32 = val.into();
            let sine = f32::sin(f);
            let f = FixedNumber::from_fix(FIX::from_num(sine));

            f
        };
        assert_eq!(expected, actual);

        let val: FixedNumber = FixedNumber::PI();
        let actual = lut.sine(val);
        let expected: FixedNumber = {
            let f: f32 = val.into();
            let sine = f32::sin(f);
            let f = FixedNumber::from_fix(FIX::from_num(sine));

            f
        };
        assert_eq!(expected, actual);

        let val: FixedNumber = FixedNumber::PI() * FixedNumber::from_i32(2);
        let actual = lut.sine(val);
        let expected: FixedNumber = {
            let f: f32 = val.into();
            let sine = f32::sin(f);
            let f = FixedNumber::from_fix(FIX::from_num(sine));

            f
        };
        assert_eq!(expected, actual);

        // TODO: fix indexing issues
        // TODO: test negative values
        // TODO: loop over all fixed number values in the range?
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_0case() {
        let lut = FixedNumberLut::new();

        // 0 case
        let theta: FixedNumber = 0.into();
        let expected: FixedNumber = 0.into();
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_halfpi_case() {
        let lut = FixedNumberLut::new();

        // 1/2 pi case
        let theta: FixedNumber = FixedNumber::PI() / 2.into();
        let expected: FixedNumber = theta;
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_neg_halfpi_case() {
        let lut = FixedNumberLut::new();

        let theta: FixedNumber = -(FixedNumber::PI() / 2.into());
        let expected: FixedNumber = FixedNumber::TWO_PI() + theta;
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_pi_case() {
        let lut = FixedNumberLut::new();

        // pi case
        let theta: FixedNumber = FixedNumber::PI();
        let expected: FixedNumber = theta;
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_neg_pi_case() {
        let lut = FixedNumberLut::new();

        let theta: FixedNumber = -(FixedNumber::PI());
        let expected: FixedNumber = FixedNumber::TWO_PI() + theta;
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_one_and_half_pi_case() {
        let lut = FixedNumberLut::new();

        // 1.5 pi case
        let theta: FixedNumber = FixedNumber::PI() + FixedNumber::PI() / 2.into();
        let expected: FixedNumber = theta;
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_neg_one_and_half_pi_case() {
        let lut = FixedNumberLut::new();

        let theta: FixedNumber = -(FixedNumber::PI() + FixedNumber::PI() / 2.into());
        let expected: FixedNumber = FixedNumber::TWO_PI() + theta;
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_two_pi_case() {
        let lut = FixedNumberLut::new();

        // 2 pi case
        let theta: FixedNumber = FixedNumber::TWO_PI();
        let expected: FixedNumber = 0.into();
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_neg_two_pi_case() {
        let lut = FixedNumberLut::new();

        let theta: FixedNumber = -(FixedNumber::TWO_PI());
        let expected: FixedNumber = 0.into();
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_two_and_half_pi_case() {
        let lut = FixedNumberLut::new();

        // 2 pi + 1/ PI case = 1/2 pi
        let theta: FixedNumber = FixedNumber::TWO_PI() + FixedNumber::PI() / 2.into();
        let expected: FixedNumber = FixedNumber::PI() / 2.into();
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_neg_two_and_half_pi_case() {
        let lut = FixedNumberLut::new();

        let theta: FixedNumber = -(FixedNumber::TWO_PI() + FixedNumber::PI() / 2.into());
        let expected: FixedNumber = FixedNumber::TWO_PI() - FixedNumber::PI() / 2.into();
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_three_pi_case() {
        let lut = FixedNumberLut::new();

        let theta: FixedNumber = FixedNumber::TWO_PI() + FixedNumber::PI();
        let expected: FixedNumber = FixedNumber::PI();
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_neg_three_pi_case() {
        let lut = FixedNumberLut::new();

        let theta: FixedNumber = -(FixedNumber::TWO_PI() + FixedNumber::PI());
        let expected: FixedNumber = FixedNumber::PI();
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_three_and_half_pi_case() {
        let lut = FixedNumberLut::new();

        let theta: FixedNumber =
            FixedNumber::TWO_PI() + FixedNumber::PI() + FixedNumber::PI() / 2.into();
        let expected: FixedNumber = FixedNumber::PI() + FixedNumber::PI() / 2.into();
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_4_pi_case() {
        let lut = FixedNumberLut::new();

        let theta: FixedNumber = FixedNumber::TWO_PI() + FixedNumber::TWO_PI();
        let expected: FixedNumber = FixedNumber::zero();
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }

    #[test]
    fn FixedNumberLookup_wrap_theta_neg_4_pi_case() {
        let lut = FixedNumberLut::new();

        let theta: FixedNumber = -(FixedNumber::TWO_PI() + FixedNumber::TWO_PI());
        let expected: FixedNumber = FixedNumber::zero();
        let actual = wrap_theta(theta);
        assert_eq!(expected, actual);
    }
}

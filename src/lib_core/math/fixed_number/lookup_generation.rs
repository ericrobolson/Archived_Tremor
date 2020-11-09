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
        let end = FixedNumber::PI() * FixedNumber::from_i32(2); // TODO: may even be able to cut LUT in half by only doing 0..PI and then reversing the indexes after a certain point?

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
        let two_pi = FixedNumber::PI() * FixedNumber::from_i32(2);
        let mut wrapped_theta = theta.remainder(two_pi);
        if wrapped_theta < 0.into() || wrapped_theta > two_pi {
            wrapped_theta = 0.into();
        }

        // Get index
        let index: usize = (wrapped_theta * self.min_val).into();
        let index: usize = index % self.sines.len();

        // Return the value
        self.sines[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

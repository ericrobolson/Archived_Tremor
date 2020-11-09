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
}

pub fn sine(theta: FixedNumber, lut: &FixedNumberLut) -> FixedNumber {
    // Map the number to a 0..2PI range
    let two_pi = FixedNumber::PI() * FixedNumber::from_i32(2);
    let mut wrapped_theta = theta.remainder(two_pi);
    if wrapped_theta < 0.into() || wrapped_theta > two_pi {
        wrapped_theta = 0.into();
    }

    // Get index
    let index: usize = (wrapped_theta * lut.min_val).into();
    let index: usize = index % lut.sines.len();

    // Return the value
    lut.sines[index]
}

pub fn generate() {
    let start = FixedNumber::zero();
    let increment = decimal_resolution_value();
    let end = FixedNumber::PI() * FixedNumber::from_i32(2);

    let mut value = start;
    let mut cosines = vec![];
    let mut sines = vec![];
    while value < end {
        let sin_lookup = f32::sin(value.into()); // TODO: convert to fixed
        sines.push(sin_lookup);

        let fix_val = FIX::from_num(sin_lookup);
        println!("VAL: {:?}", fix_val);

        let cos_lookup = f32::cos(value.into()); // TODO: convert to fixed
        cosines.push(cos_lookup);

        // Finally increment the value to calculate
        value += increment;
    }

    println!("Sines: {:?}", sines);
    println!("Sine count: {}", sines.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn FixedNumber_lookup_generation() {
        assert_eq!(FixedNumber::one(), decimal_resolution_value());
    }

    #[test]
    fn FixedNumberLookup_SinCos_Test() {
        generate();
        assert_eq!(true, false);
    }
}

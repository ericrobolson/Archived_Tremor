use crate::Number;

type Fix = i32;

implement_types!(Fix);

impl Number for Fix {
    fn i32(real_number: i32) -> Self {
        real_number as Self
    }

    fn abs(&self) -> Self {
        if *self < 0 {
            -(*self)
        } else {
            *self
        }
    }

    fn sqrt(&self) -> Self {
        unimplemented!("Need to implement fixed point sqrt")
    }

    fn sqrd(&self) -> Self {
        self * self
    }

    fn sin(&self) -> Self {
        unimplemented!("Need to implement fixed point sin")
    }

    fn cos(&self) -> Self {
        unimplemented!("Need to implement fixed point cos")
    }

    fn min(&self, other: Self) -> Self {
        if other < *self {
            other
        } else {
            *self
        }
    }

    fn max(&self, other: Self) -> Self {
        if other > *self {
            other
        } else {
            *self
        }
    }
}

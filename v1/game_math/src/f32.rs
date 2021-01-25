use crate::Number;

implement_types!(f32);


impl Number for f32{
    fn i32(real_number: i32) -> Self {
        real_number as Self
    }

    fn abs(&self) -> Self{
        f32_abs(*self)
    }

    fn sqrt(&self) -> Self {
        f32_sqrt(*self)
    }

    fn sqrd(&self) -> Self {
        self * self
    }

    fn sin(&self) -> Self {
        f32_sin(*self)
    }

    fn cos(&self) -> Self {
        f32_cos(*self)
    }

    fn min(&self, other: Self) -> Self{
        if *self < other {
            *self
        } else {
            other
        }
    }

    fn max(&self, other: Self) -> Self{
        if *self > other {
            *self
        } else {
            other
        }
    }
}

// These functions are to 'overload' the other ones and to prevent infinite recursion calls.


fn f32_abs(f: f32) -> f32{
    f.abs()
}


fn f32_cos(f: f32) -> f32{
    f.cos()
}


fn f32_sin(f: f32) -> f32{
    f.sin()
}

fn f32_sqrt(f: f32) -> f32{
    f.sqrt()
}
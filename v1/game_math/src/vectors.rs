#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Vec3<N> {
    pub x: N,
    pub y: N,
    pub z: N,
}

impl<N> Vec3<N> {
    pub fn default() -> Self {
        unimplemented!();
    }

    pub fn min(&self, other: Self) -> Self {
        unimplemented!("Componentwise min.");
    }

    pub fn max(&self, other: Self) -> Self {
        unimplemented!("Componentwise max");
    }
}

impl<R> std::ops::Mul for Vec3<R> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        unimplemented!();
        /*
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
        */
    }
}

impl<R> std::ops::Mul<R> for Vec3<R> {
    type Output = Vec3<R>;
    fn mul(self, rhs: R) -> Vec3<R> {
        unimplemented!();
        /*
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
        */
    }
}

impl<R> std::ops::MulAssign<R> for Vec3<R> {
    fn mul_assign(&mut self, rhs: R) {
        unimplemented!();
        /*
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
        */
    }
}

impl<R> std::ops::AddAssign for Vec3<R> {
    fn add_assign(&mut self, rhs: Self) {
        unimplemented!();
    }
}

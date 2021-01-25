use super::{vec3::Vec3, vec4::Vec4, Number, RawConverter};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Mat4<N>([Vec4<N>; 4])
where
    N: Number;

impl<N> Mat4<N>
where
    N: Number,
{
    /// Default Mat4 set to identity.
    pub fn default() -> Self {
        Self::identity()
    }

    /// Creates a new Mat4.
    pub fn new(m0: Vec4<N>, m1: Vec4<N>, m2: Vec4<N>, m3: Vec4<N>) -> Self {
        Self([m0, m1, m2, m3])
    }

    /// Initialize all components on the matrix's diagonal, with the remaining ones set to 0.
    pub fn i32(n: i32) -> Self {
        let o = N::i32(0);
        let i = N::i32(n);

        let m0 = Vec4::new(i, o, o, o);
        let m1 = Vec4::new(o, i, o, o);
        let m2 = Vec4::new(o, o, i, o);
        let m3 = Vec4::new(o, o, o, i);

        Self([m0, m1, m2, m3])
    }

    pub fn identity() -> Self {
        Self::i32(1)
    }

    pub fn translate(mat: Self, v: Vec3<N>) -> Self {
        unimplemented!();
    }

    pub fn scale(&self, scale: Vec3<N>) -> Self {
        unimplemented!();
    }

    pub fn inverse(&self) -> Self {
        unimplemented!();
    }
}

impl<N> RawConverter for Mat4<N>
where
    N: Number,
{
    type RawType = [[N; 4]; 4];

    fn to_raw(&self) -> Self::RawType {
        [
            self[0].to_raw(),
            self[1].to_raw(),
            self[2].to_raw(),
            self[3].to_raw(),
        ]
    }

    fn from_raw(raw: Self::RawType) -> Self {
        Self([
            Vec4::from_raw(raw[0]),
            Vec4::from_raw(raw[1]),
            Vec4::from_raw(raw[2]),
            Vec4::from_raw(raw[3]),
        ])
    }
}

impl<N> std::ops::Index<usize> for Mat4<N>
where
    N: Number,
{
    type Output = Vec4<N>;
    fn index(&self, i: usize) -> &Self::Output {
        let Mat4::<N>(mat) = self;
        &mat[i]
    }
}

impl<N> std::ops::Mul for Mat4<N>
where
    N: Number,
{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        unimplemented!();
    }
}

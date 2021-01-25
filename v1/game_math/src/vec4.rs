use super::{vec3::Vec3, Number, RawConverter};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Vec4<N>
where
    N: Copy + Clone + PartialEq,
{
    x: N,
    y: N,
    z: N,
    w: N,
}

impl<N> Vec4<N>
where
    N: Number,
{
    /// Default Vec4 set to 0.
    pub fn default() -> Self {
        Self {
            x: N::i32(0),
            y: N::i32(0),
            z: N::i32(0),
            w: N::i32(0),
        }
    }
    pub fn new(x: N, y: N, z: N, w: N) -> Self {
        Self { x, y, z, w }
    }

    pub fn i32(i: i32) -> Self {
        Self {
            x: N::i32(i),
            y: N::i32(i),
            z: N::i32(i),
            w: N::i32(i),
        }
    }

    pub fn vec3(&self) -> Vec3<N> {
        Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

impl<N> RawConverter for Vec4<N>
where
    N: Number,
{
    type RawType = [N; 4];

    fn to_raw(&self) -> Self::RawType {
        [self.x, self.y, self.z, self.w]
    }

    fn from_raw(raw: Self::RawType) -> Self {
        Self::new(raw[0], raw[1], raw[2], raw[3])
    }
}

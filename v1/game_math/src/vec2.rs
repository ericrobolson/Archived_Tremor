use super::*;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Vec2<N>
where
    N: Number,
{
    pub x: N,
    pub y: N,
}

impl<N> Vec2<N>
where
    N: Number,
{
    pub fn default() -> Self {
        Self {
            x: N::i32(0),
            y: N::i32(0),
        }
    }

    pub fn new(x: N, y: N) -> Self {
        Self { x, y }
    }
}

impl<N> RawConverter for Vec2<N>
where
    N: Number,
{
    type RawType = [N; 2];

    fn to_raw(&self) -> Self::RawType {
        [self.x, self.y]
    }

    fn from_raw(raw: Self::RawType) -> Self {
        Self::new(raw[0], raw[1])
    }
}

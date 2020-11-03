use crate::lib_core::math::{FixedNumber, Vec3};

#[derive(Copy, Clone, Debug)]
pub struct Transformation {
    pub translation: Vec3,
    // Rotation in radians
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl Transformation {
    pub fn default() -> Self {
        Self::new((0, 0, 0).into(), (0, 0, 0).into(), (1, 1, 1).into())
    }

    pub fn new(translation: Vec3, rotation: Vec3, scale: Vec3) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    pub fn apply_parent(&self, parent: &Self) -> Self {
        Self {
            translation: self.translation + parent.translation,
            rotation: self.rotation + parent.rotation,
            scale: self.scale * parent.scale,
        }
    }
}

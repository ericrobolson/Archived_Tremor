use crate::lib_core::math::{FixedNumber, Vec3};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Transform {
    pub position: Vec3,
    // Rotation in radians
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl Transform {
    pub fn default() -> Self {
        Self::new((0, 0, 0).into(), (0, 0, 0).into(), (1, 1, 1).into())
    }

    pub fn new(position: Vec3, rotation: Vec3, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    pub fn apply_parent(&self, parent: &Self) -> Self {
        Self {
            position: self.position + parent.position,
            rotation: self.rotation + parent.rotation,
            scale: self.scale * parent.scale,
        }
    }
}

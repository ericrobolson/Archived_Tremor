use super::{boundingbox::BoundingBox, material::Material};
use game_math::f32::*;

pub struct Primitive {
    pub first_index: u32,
    pub index_count: u32,
    pub vertex_count: u32,
    pub bounding_box: BoundingBox,
    pub material: Material,
}

impl Primitive {
    pub fn set_bounding_box(&mut self, min: Vec3, max: Vec3) {
        self.bounding_box.min = min;
        self.bounding_box.max = max;
        self.bounding_box.valid = true;
    }
}

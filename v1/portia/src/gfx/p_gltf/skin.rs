use game_math::f32::*;
use std::rc::Rc;

use super::node::Node;

pub struct Skin {
    name: String,
    skeleton_root: Option<usize>,
    pub inverse_bind_matrices: Vec<Mat4>,
    pub joints: Vec<usize>,
}

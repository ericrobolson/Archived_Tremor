use super::{node::Node, SmartPointer};
use game_math::f32::*;

pub enum PathType {
    Translation,
    Rotation,
    Scale,
}
pub struct AnimationChannel {
    path: PathType,
    node: usize,
    sampler_index: u32,
}

pub enum InterpolationType {
    Linear,
    Step,
    CubicSpline,
}
pub struct AnimationSampler {
    interpolation: InterpolationType,
    inputs: Vec<f32>,
    outputs: Vec<Vec4>,
}
pub struct Animation {
    name: String,
    samplers: Vec<AnimationSampler>,
    channels: Vec<AnimationChannel>,
    start: f32,
    end: f32,
}
impl Animation {
    pub fn new(name: String, samplers: Vec<AnimationSampler>) -> Self {
        let start = f32::MAX;
        let end = f32::MIN;
        Self {
            start,
            end,
            name,
            samplers,
            channels: vec![],
        }
    }
}

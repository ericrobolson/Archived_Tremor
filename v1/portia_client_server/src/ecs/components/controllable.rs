use crate::ecs::prelude::*;
use crate::math::*;

// TODO: this is the 'input' that controls an actor. Used for all ghostable/creatable objects.
pub struct Controllable {
    yaw: Num,
    pitch: Num,
    roll: Num,
}

impl Component for Controllable {
    fn default(world_settings: &WorldSettings) -> Self {
        Self {
            yaw: 0.,
            pitch: 0.,
            roll: 0.,
        }
    }
}

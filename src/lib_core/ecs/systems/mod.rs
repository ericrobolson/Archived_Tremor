use super::*;

use crate::lib_core::spatial::physics::CollisionShape;

//TODO: Determine and design pattern for systems. The components it will use, the component it requires, the entities it needs.

mod physics;
pub use physics::Physics;
pub use physics::VerletParticleSystem;

mod input_actions;
pub use input_actions::InputActions;

pub trait System {
    fn new(entity_count: usize) -> Self;
    fn reset(&mut self);
    fn dispatch(world: &mut World);
    fn cleanup(world: &mut World);
}

pub fn camera_movement(world: &mut World) {
    let mut min_pos = Vec3::new();
    let mut max_pos = Vec3::new();

    // TODO: change to keep all targets in view. Right now it just tracks

    const MASK: MaskType = mask!(Mask::TRANSFORM, Mask::TRACKABLE);
    for entity in matching_entities!(world, MASK) {
        let pos = world.transforms[entity].position;
        if min_pos.x > pos.x {
            min_pos.x = pos.x;
        }
        if min_pos.y > pos.y {
            min_pos.y = pos.y;
        }
        if min_pos.z > pos.z {
            min_pos.z = pos.z;
        }
        //
        if max_pos.x < pos.x {
            max_pos.x = pos.x;
        }
        if max_pos.y < pos.y {
            max_pos.y = pos.y;
        }
        if max_pos.z < pos.z {
            max_pos.z = pos.z;
        }

        max_pos = world.transforms[entity].position;
    }

    return;

    // need to actually calculate the camera position + target

    let delta = max_pos - min_pos;
    world.camera.target = Vec3 {
        x: delta.x / 2.into(),
        y: delta.y / 2.into(),
        z: delta.z / 2.into(),
    };
}

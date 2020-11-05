use super::*;

pub fn input_register(world: &mut World) {}
pub fn input_actions(world: &mut World) {
    const INPUT_MOVE_MASK: MaskType = Mask::VELOCITY & Mask::PLAYER_INPUT;
    for entity in world
        .masks
        .iter()
        .enumerate()
        .filter(|(i, mask)| **mask & INPUT_MOVE_MASK == INPUT_MOVE_MASK)
        .map(|(i, mask)| i)
    {
        // TODO: change 'actionX' to actual input name
        let move_speed = FixedNumber::fraction(40.into());
        if world.inputs[entity].up {
            world.velocities[entity].position.y += move_speed;
        } else if world.inputs[entity].down {
            world.velocities[entity].position.y -= move_speed;
        }

        if world.inputs[entity].left {
            world.velocities[entity].position.x -= move_speed;
        } else if world.inputs[entity].right {
            world.velocities[entity].position.x += move_speed;
        }
    }
}

pub fn collision_detection(world: &mut World) {}

pub fn collision_resolution(world: &mut World) {}

pub fn movement(world: &mut World) {
    {
        const MASK: MaskType = Mask::TRANSFORM & Mask::VELOCITY;
        for entity in world
            .masks
            .iter()
            .enumerate()
            .filter(|(i, mask)| **mask & MASK == MASK)
            .map(|(i, mask)| i)
        {
            world.transforms[entity].position += world.velocities[entity].position;
            world.transforms[entity].rotation += world.velocities[entity].rotation;
            // TODO: scale? Not for now.
        }
    }
}

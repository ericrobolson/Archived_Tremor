use super::*;

pub struct InputActions {}
impl System for InputActions {
    fn new(max_entities: usize) -> Self {
        Self {}
    }

    fn reset(&mut self) {}
    fn dispatch(world: &mut World) {
        input_actions(world);
    }
    fn cleanup(world: &mut World) {}
}

fn input_actions(world: &mut World) {
    const INPUT_MOVE_MASK: MaskType = mask!(Mask::VELOCITY, Mask::PLAYER_INPUT);

    for entity in matching_entities!(world, INPUT_MOVE_MASK) {
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
        } else {
            // world.velocities[entity].position.x = 0.into();
        }
    }
}

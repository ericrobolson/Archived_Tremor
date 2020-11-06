use super::*;

pub fn input_register(world: &mut World) {}
pub fn input_actions(world: &mut World) {
    const INPUT_MOVE_MASK: MaskType = Mask::VELOCITY | Mask::PLAYER_INPUT;
    for entity in world
        .masks
        .iter()
        .enumerate()
        .filter(|(i, mask)| **mask & INPUT_MOVE_MASK == INPUT_MOVE_MASK)
        .map(|(i, mask)| i)
    {
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

pub fn force_application(world: &mut World) {
    return;
    const MASK: MaskType = Mask::BODY | Mask::VELOCITY;
    for entity in world
        .masks
        .iter()
        .enumerate()
        .filter(|(i, mask)| **mask & MASK == MASK)
        .map(|(i, mask)| i)
        .collect::<Vec<Entity>>()
    {
        let body_type = world.bodies[entity];

        if body_type == PhysicBodies::Kinematic || body_type == PhysicBodies::Rigidbody {
            world.add_component(entity, Mask::FORCE).unwrap();
            world.forces[entity].position.y = -FixedNumber::fraction(20.into());
        }
    }
}

pub fn collision_detection(world: &mut World) {
    const MASK: MaskType = Mask::TRANSFORM | Mask::AABB | Mask::BODY;
    let entities = world
        .masks
        .iter()
        .enumerate()
        .filter(|(i, mask)| **mask & MASK == MASK)
        .map(|(i, mask)| i)
        .collect::<Vec<Entity>>();

    // Only proceed if there's at least 2 entities
    if entities.len() >= 2 {
        // Iterate over all entity pairs
        for entity1 in 0..entities.len() - 1 {
            for entity2 in (entity1 + 1)..entities.len() {
                let entity1 = entities[entity1];
                let entity2 = entities[entity2];

                if entity1 == entity2 {
                    continue;
                }

                let aabb1 = &world.aabbs[entity1];
                let aabb2 = &world.aabbs[entity2];

                let transform1 = &world.transforms[entity1];
                let transform2 = &world.transforms[entity2];

                // TODO: heirarchies
                // TODO: rotations
                if aabb1.colliding(transform1, aabb2, transform2) {
                    world.add_component(entity1, Mask::COLLISIONS).unwrap();
                    world.add_component(entity2, Mask::COLLISIONS).unwrap();

                    // TODO: link up manifolds

                    world.collision_lists[entity1].add(entity2);
                    world.collision_lists[entity2].add(entity1);
                }
            }
        }
    }
}

pub fn collision_resolution(world: &mut World) {
    const MASK: MaskType = Mask::TRANSFORM | Mask::BODY | Mask::COLLISIONS;
    let entities = world
        .masks
        .iter()
        .enumerate()
        .filter(|(i, mask)| **mask & MASK == MASK)
        .map(|(i, mask)| i)
        .collect::<Vec<Entity>>();

    for entity in entities {
        for collision in &world.collision_lists[entity].collisions() {
            // TODO: how to resolve?
            println!("Resolved collision");
        }

        // Remove and reset all collisions so next frame is 'clean'
        world.collision_lists[entity].reset();
        world.remove_component(entity, Mask::COLLISIONS).unwrap();
    }
}

pub fn movement(world: &mut World) {
    {
        const MASK: MaskType = Mask::TRANSFORM | Mask::VELOCITY;
        for entity in world
            .masks
            .iter()
            .enumerate()
            .filter(|(i, mask)| **mask & MASK == MASK)
            .map(|(i, mask)| i)
            .collect::<Vec<Entity>>()
        {
            if world.has_component(entity, Mask::FORCE) {
                world.velocities[entity].position += world.forces[entity].position;
                world.velocities[entity].rotation += world.forces[entity].rotation;
            }
            world.remove_component(entity, Mask::FORCE).unwrap();

            world.transforms[entity].position += world.velocities[entity].position;
            world.transforms[entity].rotation += world.velocities[entity].rotation;

            // TODO: scale? Not for now.
        }
    }
}

pub fn camera_movement(world: &mut World) {
    let mut min_pos = Vec3::new();
    let mut max_pos = Vec3::new();

    // TODO: change to keep all targets in view. Right now it just tracks

    const MASK: MaskType = Mask::TRANSFORM | Mask::TRACKABLE;
    for entity in world
        .masks
        .iter()
        .enumerate()
        .filter(|(i, mask)| **mask & MASK == MASK)
        .map(|(i, mask)| i)
    {
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

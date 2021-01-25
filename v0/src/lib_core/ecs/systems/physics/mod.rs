use super::*;

mod verlet;
pub use verlet::VerletParticleSystem;

pub struct Physics {}

impl System for Physics {
    fn new(max_entities: usize) -> Self {
        Self {}
    }

    fn reset(&mut self) {}
    fn dispatch(world: &mut World) {
        force_application(world);
        movement(world);
        collision_detection(world);
        collision_resolution(world);
    }

    fn cleanup(world: &mut World) {
        // Remove and reset all collisions so next frame is 'clean'
        const MASK: MaskType = mask!(Mask::COLLISIONS,);
        for entity in matching_entities!(world, MASK).collect::<Vec<Entity>>() {
            world.collision_lists[entity].reset();
            world.remove_component(entity, Mask::COLLISIONS).unwrap();
        }
    }
}

fn movement(world: &mut World) {
    const MASK: MaskType = mask!(Mask::TRANSFORM, Mask::VELOCITY);
    for entity in matching_entities!(world, MASK).collect::<Vec<Entity>>() {
        if world.has_component(entity, Mask::FORCE) {
            world.velocities[entity].position += world.forces[entity].position;
            world.velocities[entity].rotation *= world.forces[entity].rotation;
        }
        //world.remove_component(entity, Mask::FORCE).unwrap();

        world.transforms[entity].position += world.velocities[entity].position;
        world.transforms[entity].rotation *= world.velocities[entity].rotation;

        // TODO: scale? Not for now.
    }
}

fn force_application(world: &mut World) {
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
            world.velocities[entity].position.y -= FixedNumber::fraction(1000.into());
        }
    }
}

fn collision_detection(world: &mut World) {
    // For every component that has a collision primitive, use the latest transform and convert them from local space to world space.
    // TODO: how to handle hierarchies + rotations n stuff?
    const TRANSFORM_UPDATE: MaskType = mask!(Mask::TRANSFORM, Mask::COLLISION_SHAPE);
    for entity in matching_entities!(world, TRANSFORM_UPDATE).collect::<Vec<Entity>>() {
        let world_transform = world.transforms[entity];
        world.collision_shapes[entity] = match world.collision_shapes[entity] {
            CollisionShapes::Circle(mut sphere) => {
                sphere.update_transform(world_transform);

                CollisionShapes::Circle(sphere)
            }
            CollisionShapes::Capsule(mut capsule) => {
                capsule.update_transform(world_transform);

                CollisionShapes::Capsule(capsule)
            }
            CollisionShapes::Aabb(aabb) => CollisionShapes::Aabb(aabb),
        };
    }

    // Calculate the collisions
    const MASK: MaskType = mask!(Mask::TRANSFORM, Mask::COLLISION_SHAPE);

    let entities = matching_entities!(world, MASK).collect::<Vec<Entity>>();

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

                let shape1 = &world.collision_shapes[entity1];
                let shape2 = &world.collision_shapes[entity2];

                let transform1 = &world.transforms[entity1];
                let transform2 = &world.transforms[entity2];

                match shape1.colliding(shape2) {
                    Some(manifold) => {
                        world.add_component(entity1, Mask::COLLISIONS).unwrap();
                        world.add_component(entity2, Mask::COLLISIONS).unwrap();

                        let mut manifold2 = manifold;
                        manifold2.normal = -manifold.normal;

                        world.collision_lists[entity1].add(entity2, manifold);
                        world.collision_lists[entity2].add(entity1, manifold2);
                    }
                    None => {}
                }
            }
        }
    }
}

fn collision_resolution(world: &mut World) {
    const MASK: MaskType = mask!(Mask::TRANSFORM, Mask::BODY, Mask::COLLISIONS);
    let entities = matching_entities!(world, MASK).collect::<Vec<Entity>>();

    let mut position_adjustments: Vec<(Entity, Vec3)> = vec![];
    let mut velocities_to_update: Vec<(Entity, Vec3)> = vec![];

    for entity in entities {
        for collision in &world.collision_lists[entity].collisions() {
            if world.bodies[entity] == PhysicBodies::Static {
                continue;
            }

            let other_entity = collision.other_entity;
            let velocity1 = world.velocities[entity];
            let velocity2 = world.velocities[other_entity];

            // Calculate velocities
            {
                let relative_velocity = velocity1.position - velocity2.position;
                let velocity_along_normal = relative_velocity.dot(collision.manifold.normal);

                // Calculate impulse scalar
                //TODO: add in rotations?
                let entity_restitution: FixedNumber = world.voxel_chunks[entity].restitution();
                let other_entity_restitution: FixedNumber =
                    world.voxel_chunks[other_entity].restitution();
                let restitution = FixedNumber::min(entity_restitution, other_entity_restitution);

                let inverse_entity_mass: FixedNumber = world.voxel_chunks[entity].inv_mass();
                let inverse_other_mass: FixedNumber = {
                    if world.bodies[other_entity] == PhysicBodies::Static {
                        FixedNumber::decimal_resolution_value()
                    } else {
                        world.voxel_chunks[other_entity].inv_mass()
                    }
                };

                let impulse_magnitude = (restitution + 1.into()) * velocity_along_normal
                    / (inverse_entity_mass + inverse_other_mass);

                let impulse = collision.manifold.normal * impulse_magnitude; // Apply normal
                let impulse = impulse * inverse_entity_mass; // Now scale it by mass

                velocities_to_update.push((entity, impulse));

                // Adjust positions so they aren't colliding anymore
                position_adjustments.push((
                    entity,
                    collision.manifold.normal * collision.manifold.penetration,
                ));
            }
        }
    }

    // Move entities so they're not colliding anymore.
    for (entity, position_adjustment) in position_adjustments {
        // TODO: rotations

        world.transforms[entity].position -= position_adjustment;
    }

    // Now that we've calculated all the velocities and updates, apply them.
    for (entity, velocity_pos_update) in velocities_to_update {
        world.velocities[entity].position -= velocity_pos_update;
        // TODO: rotations
    }
}

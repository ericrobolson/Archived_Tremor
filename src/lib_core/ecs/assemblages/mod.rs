use super::*;

pub fn assemble_sphere_shape(
    entity: Entity,
    transform: Transform,
    velocity: Transform,
    world: &mut World,
) -> Result<(), String> {
    // Voxels
    world.add_component(entity, Mask::TRANSFORM)?;
    world.transforms[entity] = transform;

    world.add_component(entity, Mask::VELOCITY)?;
    world.velocities[entity] = velocity;

    world.add_component(entity, Mask::BODY)?;
    world.bodies[entity] = PhysicBodies::Kinematic;

    let mut sphere = Sphere::new(10.into(), Transform::default());

    // Init chunk from capsule
    world.add_component(entity, Mask::VOXEL_CHUNK)?;

    let radius: usize = sphere.radius.into();
    let len = radius * 2;
    world.voxel_chunks[entity] = Chunk::new(len, len, len, 2);
    let (x_depth, y_depth, z_depth) = world.voxel_chunks[entity].capacity();
    let chunk = &mut world.voxel_chunks[entity];

    // Cast the capsule to voxel space
    for x in 0..x_depth {
        for y in 0..y_depth {
            chunk.set_voxel(x, y, 0, Voxel::Metal);

            for z in 0..z_depth {
                let point = Vec3 {
                    x: x.into(),
                    y: y.into(),
                    z: z.into(),
                };

                if sphere.contains_point(point) {
                    chunk.set_voxel(x, y, z, Voxel::Cloth);
                }
            }
        }
    }

    // Put the capsule back into world space
    sphere.update_transform(world.transforms[entity]);
    world.add_component(entity, Mask::COLLISION_SHAPE)?;
    world.collision_shapes[entity] = CollisionShapes::Circle(sphere);

    Ok(())
}

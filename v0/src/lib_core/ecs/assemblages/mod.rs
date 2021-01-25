use super::*;
use crate::lib_core::spatial::physics::CollisionShape;

pub fn assemble_capsule_shape(
    entity: Entity,
    transform: Transform,
    velocity: Transform,
    body_type: PhysicBodies,
    capsule_radius: FixedNumber,
    capsule_length: FixedNumber,
    world: &mut World,
) -> Result<(), String> {
    // Voxels
    world.add_component(entity, Mask::VOXEL_CHUNK)?;
    world.add_component(entity, Mask::TRANSFORM)?;
    world.add_component(entity, Mask::BODY)?;
    world.bodies[entity] = body_type;

    world.transforms[entity] = transform;
    world.set_position(entity, transform.position);

    world.add_component(entity, Mask::VELOCITY)?;
    world.velocities[entity] = velocity;

    world.add_component(entity, Mask::COLLISION_SHAPE)?;

    let mut capsule = Capsule::new(capsule_radius, capsule_length, Transform::default());

    // Init chunk from capsule
    let radius: usize = capsule.radius.into();
    let len: usize = capsule.length.into();
    let len = radius * 2 + len; // Need to account for the end circles
    world.voxel_chunks[entity] = Chunk::new(len, 2 * radius, 2 * radius, 2);
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

                if capsule.contains_point(point) {
                    chunk.set_voxel(x, y, z, Voxel::Bone);
                }
            }
        }
    }

    // Put the capsule back into world space
    capsule.update_transform(world.transforms[entity]);
    world.collision_shapes[entity] = CollisionShapes::Capsule(capsule);

    Ok(())
}

pub fn assemble_sphere_shape(
    entity: Entity,
    transform: Transform,
    velocity: Transform,
    voxel: Voxel,
    world: &mut World,
) -> Result<(), String> {
    // Voxels
    world.add_component(entity, Mask::TRANSFORM)?;
    world.transforms[entity] = transform;

    world.set_position(entity, transform.position);

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
                    chunk.set_voxel(x, y, z, voxel);
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

pub fn assemble_box_shape(
    entity: Entity,
    transform: Transform,
    velocity: Transform,
    voxel: Voxel,
    world: &mut World,
) -> Result<(), String> {
    unimplemented!();
    /*
     match self.add_entity() {
                        Some(entity) => {
                            // Voxels
                            self.add_component(entity, Mask::VOXEL_CHUNK)?;
                            self.add_component(entity, Mask::TRANSFORM)?;
                            self.add_component(entity, Mask::BODY)?;
                            self.bodies[entity] = PhysicBodies::Static;


                            self.transforms[entity] = Transform::new((-100,-10,0).into(), Quaternion::default(), Vec3::one());

                            self.voxel_chunks[entity] = Chunk::new(200, 1, 40, 2);

                            let (x_depth, y_depth, z_depth) = self.voxel_chunks[entity].capacity();

                            let chunk = &mut self.voxel_chunks[entity];

                            for x in 0..x_depth{
                                for z in 0..z_depth {
                                    chunk.set_voxel(x,0,z, Voxel::Metal);
                                }
                            }

                            self.add_component(entity, Mask::COLLISION_SHAPE)?;

                            let max_aabb = self.transforms[entity].scale * Vec3{x: x_depth.into(), y: y_depth.into(), z: z_depth.into()};

                            self.collision_shapes[entity] = CollisionShapes::Aabb (Aabb{
                                min: Vec3::new(),
                                max: max_aabb
                            });


                        }
                        None => {}
                    }

    Ok(())
    */
}

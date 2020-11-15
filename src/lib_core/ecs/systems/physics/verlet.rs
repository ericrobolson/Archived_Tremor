use super::*;

pub struct VerletParticleSystem {
    sim: VerletParticleSimulation,
}

impl System for VerletParticleSystem {
    fn new(max_entities: usize) -> Self {
        Self {
            sim: VerletParticleSimulation::new(max_entities),
        }
    }

    fn reset(&mut self) {}
    fn dispatch(world: &mut World) {
        // Update the particle simulator
        {
            let delta_t = world.delta_t();
            let sim = &mut world.verlet_simulation.sim;
            sim.time_step = delta_t;
        }

        // Execute the particle simulator
        VerletParticleSimulation::step(world);
    }

    fn cleanup(world: &mut World) {
        // Reset forces

        const MASK: MaskType = mask!(Mask::TRANSFORM, Mask::BODY);
        for entity in matching_entities!(world, MASK).collect::<Vec<Entity>>() {
            world.forces[entity].position = (0, 0, 0).into();
        }
    }
}

struct VerletParticleSimulation {
    gravity: Vec3,
    time_step: FixedNumber, //TODO: calculate this
}

impl VerletParticleSimulation {
    pub fn new(max_entities: usize) -> Self {
        let mut gravity: Vec3 = (0, 0, 0).into();
        gravity.y = -FixedNumber::fraction(200.into());
        Self {
            gravity,
            time_step: 0.into(),
        }
    }

    pub fn step(world: &mut World) {
        Self::accumulate_forces(world);
        Self::verlet(world);
        Self::satisfy_constraints(world);
    }

    fn verlet(world: &mut World) {
        const MASK: MaskType = mask!(Mask::TRANSFORM, Mask::BODY);
        for entity in matching_entities!(world, MASK).collect::<Vec<Entity>>() {
            let temp = world.transforms[entity].position;
            let old_pos = world.prev_position[entity];
            let acceleration = world.forces[entity].position;

            let new_pos = (temp - old_pos) + acceleration * world.delta_t().sqrd();

            world.set_position(entity, temp + new_pos);
        }
    }

    fn satisfy_constraints(world: &mut World) {
        const NUM_ITERATIONS: usize = 1;
        for _ in 0..NUM_ITERATIONS {
            // Simple box constraint applied to all entities
            {
                let half_box_size: FixedNumber = 250.into();
                let box_min: Vec3 = (-half_box_size, -half_box_size, -half_box_size).into();
                let box_max: Vec3 = (half_box_size, half_box_size, half_box_size).into();

                const MASK: MaskType = mask!(Mask::TRANSFORM, Mask::BODY);
                for entity in matching_entities!(world, MASK).collect::<Vec<Entity>>() {
                    satisfy_box_constraint(entity, world, box_min, box_max);
                }
            }

            // Line constraints
            // Dummy hard coded constraint
            // TODO: replace with programatic constraints
            let player_entity = 3;
            let sphere_entity = 2;
            let line_dist: FixedNumber = 100.into();
            satisfy_line_constraint(sphere_entity, player_entity, line_dist, world)
        }
    }

    fn accumulate_forces(world: &mut World) {
        const MASK: MaskType = mask!(Mask::TRANSFORM, Mask::BODY);
        for entity in matching_entities!(world, MASK).collect::<Vec<Entity>>() {
            let acceleration = world.verlet_simulation.sim.gravity;

            world.forces[entity].position += acceleration;
        }
    }
}

fn satisfy_box_constraint(entity: Entity, world: &mut World, box_min: Vec3, box_max: Vec3) {
    let pos = world.transforms[entity].position;

    let min_pos = pos.componentwise_max(box_min);
    let pos = min_pos.componentwise_min(box_max);

    world.transforms[entity].position = pos;
}

fn satisfy_line_constraint(
    entity1: Entity,
    entity2: Entity,
    distance: FixedNumber,
    world: &mut World,
) {
    let pos1 = world.transforms[entity1].position;
    let pos2 = world.transforms[entity2].position;

    println!("Positions: {:?}, {:?}", pos1, pos2);

    let delta = pos1 - pos2;

    println!("delta: {:?}", delta);

    let delta_len = delta.len(); // Explodes here when pos2 - pos1. Not sure why, maybe has to do with ordering? TODO: remove sqrt for len
    println!("delta_len: {:?}", delta_len);

    let diff = (delta_len - distance) / delta_len;

    let modifier = delta * diff * FixedNumber::fraction(2.into());

    println!("diff: {:?}", diff);
    println!("mod: {:?}", modifier);

    world.transforms[entity1].position -= modifier;
    world.transforms[entity2].position += modifier;
}

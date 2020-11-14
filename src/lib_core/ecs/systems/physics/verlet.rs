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
        //TODO: add in entities + positions to sim

        // Execute the particle simulator
        {
            let delta_t = world.delta_t();
            let sim = &mut world.verlet_simulation.sim;
            sim.time_step = delta_t;
            sim.step();
        }

        //TODO: write back positions + components from sim
    }

    fn cleanup(world: &mut World) {}
}

struct VerletParticleSimulation {
    positions: Vec<Vec3>,
    old_positions: Vec<Vec3>,
    accelerations: Vec<Vec3>,
    gravity: Vec3,
    time_step: FixedNumber, //TODO: calculate this
}

impl VerletParticleSimulation {
    pub fn new(max_entities: usize) -> Self {
        Self {
            positions: Vec::with_capacity(max_entities),
            old_positions: Vec::with_capacity(max_entities),
            accelerations: Vec::with_capacity(max_entities),
            gravity: (0, -1, 0).into(),
            time_step: 0.into(),
        }
    }
    pub fn step(&mut self) {
        self.accumulate_forces();
        self.verlet();
        self.satisfy_constraints();
    }

    fn verlet(&mut self) {
        for i in 0..self.positions.len() {
            let temp = self.positions[i];
            let old_pos = self.old_positions[i];
            let acceleration = self.accelerations[i];

            self.positions[i] += temp - old_pos + acceleration * self.time_step * self.time_step;
            self.old_positions[i] = temp;
        }
    }

    fn satisfy_constraints(&mut self) {
        // Simple box constraint
        let box_min: Vec3 = (0, 0, 0).into();
        let box_max: Vec3 = (1000, 1000, 1000).into();

        for i in 0..self.positions.len() {
            let pos = self.positions[i];

            let min_pos = pos.componentwise_max(box_min);
            self.positions[i] = min_pos.componentwise_min(box_max);
        }
    }

    fn accumulate_forces(&mut self) {
        for acceleration in self.accelerations.iter_mut() {
            acceleration.x = self.gravity.x;
            acceleration.y = self.gravity.y;
            acceleration.z = self.gravity.z;
        }
    }
}

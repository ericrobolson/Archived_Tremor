pub mod particle;
use particle::*;

mod numbers;
use numbers::*;

pub type ParticleId = usize;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Constraint<N>
where
    N: Number,
{
    Stick {
        particle1: ParticleId,
        particle2: ParticleId,
        rest_length: N,
        rest_len_sqrd: Option<N>,
    },
}

/// Particle system for physics based on Verlet Integration scheme
#[derive(Clone, Debug, PartialEq)]
pub struct ParticleSystem<N>
where
    N: Number,
{
    pub particles: Vec<Particle<N>>,
    constraints: Vec<Constraint<N>>,
    gravity: Vec3<N>,
    timestep: N,
    max_particles: usize,
}

impl<N> ParticleSystem<N>
where
    N: Number,
{
    pub fn new(max_particles: usize, gravity: Vec3<N>) -> Self {
        let sim = Self {
            max_particles,
            gravity,
            particles: Vec::with_capacity(max_particles),
            constraints: Vec::with_capacity(max_particles),
            timestep: N::i32(1),
        };

        sim
    }

    pub fn add_constraint(&mut self, constraint: Constraint<N>) {
        // TODO: if Line constraint, precalculate rest_len_sqrd
        self.constraints.push(constraint);
    }

    pub fn add_entity(
        &mut self,
        position: Vec3<N>,
        inv_mass: N,
        velocity: Option<Vec3<N>>,
    ) -> ParticleId {
        let velocity = match velocity {
            Some(v) => v,
            None => Vec3::default(),
        };

        let p = Particle::new(position, inv_mass, velocity);
        let id = self.particles.len();
        self.particles.push(p);

        id
    }

    pub fn particles(&self) -> &Vec<Particle<N>> {
        &self.particles
    }

    /// Step the physics simulation
    pub fn timestep(&mut self) {
        self.accumulate_forces();
        self.verlet();
        self.satisfy_constraints(20);
    }

    /// Integrate verlet
    fn verlet(&mut self) {
        for particle in self.particles.iter_mut() {
            let current_pos = particle.position;
            let temp = current_pos;
            let old = particle.old_position;
            let acceleration = particle.force;
            particle.position += current_pos - old + acceleration * self.timestep * self.timestep;
            particle.old_position = temp;
        }
    }

    /// Add forces
    fn accumulate_forces(&mut self) {
        for particle in self.particles.iter_mut() {
            particle.force = self.gravity;
        }
    }

    fn satisfy_constraints(&mut self, constraint_iterations: usize) {
        for iteration in 0..constraint_iterations {
            // Testing bounding box
            for particle in self.particles.iter_mut() {
                {
                    let min = N::i32(-8);
                    let max = N::i32(8);
                    let min = Vec3::new(min, min, min);
                    let max = Vec3::new(max, max, max);

                    particle.position = particle.position.max(min).min(max);
                }
            }

            // Iterate over constraints
            let two = N::i32(2);

            for constraint in self.constraints.iter() {
                match constraint {
                    Constraint::Stick {
                        particle1,
                        particle2,
                        rest_length,
                        rest_len_sqrd,
                    } => {
                        let rest_len_sqrd = match rest_len_sqrd {
                            Some(r) => *r,
                            None => {
                                let r = *rest_length;
                                r * r
                            }
                        };

                        let particle1 = *particle1;
                        let particle2 = *particle2;
                        let p1 = self.particles[particle1];
                        let p2 = self.particles[particle2];

                        let delta = p1.position - p2.position;
                        let delta_len = delta.dot(delta).sqrt();
                        let diff = (delta_len - *rest_length) / delta_len;

                        let mut delta: Vec3<N> = p1.position - p2.position;
                        delta *= (rest_len_sqrd) / (delta.dot(delta) + rest_len_sqrd);

                        // NOTE: I had to reverse the add/sub ops here. Not sure why, but may play a role later on.
                        self.particles[particle1].position -= delta * p1.inv_mass * diff / two;
                        self.particles[particle2].position += delta * p2.inv_mass * diff / two;
                    }
                }
            }

            // TESTING: two locked particle for cloth
            {
                self.particles[0].position = Vec3::default();
                self.particles[8].position =
                    Vec3::default() + Vec3::new(N::i32(2), N::i32(0), N::i32(0));
            }
        }
    }
}

fn apply_stick_constraint<N>(
    particle1: ParticleId,
    particle2: ParticleId,
    rest_len: N,
    sim: &mut ParticleSystem<N>,
) where
    N: Number,
{
}

#[cfg(test)]
mod tests {
    use super::{Constraint, Particle, ParticleSystem};
    use game_math::f32::*;

    #[test]
    fn ParticleSystem_new_sets_values_properly() {
        let system = ParticleSystem::new(2, Vec3::new(0.0, -1.0, 0.0));
        assert_eq!(Vec3::new(0.0, -1.0, 0.0), system.gravity);
        assert_eq!(1.0, system.timestep);
        assert_eq!(2, system.max_particles);
    }

    #[test]
    fn ParticleSystem_single_particle_applies_gravity_and_moves() {
        let mut system = ParticleSystem::new(1, Vec3::new(0.0, -1.0, 0.0));

        system.add_entity(Vec3::new(0.0, 1.0, 0.0), None);

        system.timestep();

        for particle in system.particles() {
            assert_eq!(Vec3::new(0.0, 0.0, 0.0), particle.position);
            assert_eq!(Vec3::new(0.0, 1.0, 0.0), particle.old_position);
            assert_eq!(system.gravity, particle.force);
        }
    }

    #[test]
    fn ParticleSystem_single_particle_applies_stick_constraint() {
        let mut system = ParticleSystem::new(1, Vec3::new(0.0, -1.0, 0.0));

        let a = 2.;
        let b = 3.;
        let c = -5.;

        let p1 = system.add_entity(Vec3::new(a, a, c), None);

        let d = a / 100.;
        let p2 = system.add_entity(Vec3::new(b, a, c), Some(Vec3::new(d, d, d)));

        system.add_constraint(Constraint::Stick {
            particle1: p1,
            particle2: p2,
            rest_length: 1.,
            rest_len_sqrd: None,
        });

        system.timestep();

        let particle = system.particles[0];
        assert_eq!(Vec3::new(2.0101922, 1.0001998, -4.9998), particle.position);
        assert_eq!(Vec3::new(2.0, 2.0, -5.0), particle.old_position);
        assert_eq!(system.gravity, particle.force);

        let particle = system.particles[1];
        assert_eq!(Vec3::new(3.0098078, 1.0198002, -4.9802), particle.position);
        assert_eq!(Vec3::new(3.0, 2.0, -5.0), particle.old_position);
        assert_eq!(system.gravity, particle.force);
    }
}

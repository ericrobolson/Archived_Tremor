use crate::numbers::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Particle<N>
where
    N: Number,
{
    pub inv_mass: N,
    pub position: Vec3<N>,
    pub old_position: Vec3<N>,
    pub force: Vec3<N>,
}

impl<N> Particle<N>
where
    N: Number,
{
    /// Creates a new particle. inv_mass = 0 is 'infinite mass' or totally immovable.
    pub fn new(position: Vec3<N>, inv_mass: N, velocity: Vec3<N>) -> Self {
        Self {
            inv_mass,
            position: position,
            old_position: position - velocity,
            force: Vec3::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Particle;
    use game_math::f32::*;

    #[test]
    fn Particle_new() {
        let position = Vec3::new(1.0, 2.0, 3.0);
        let velocity = Vec3::new(1.0, 1.0, 1.0);

        let particle = Particle::new(position, velocity);

        assert_eq!(position, particle.position);
        assert_eq!(Vec3::default(), particle.force);
        assert_eq!(position - velocity, particle.old_position);
    }
}

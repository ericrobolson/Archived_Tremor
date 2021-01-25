use crate::lib_core::{
    ecs::Entity,
    math::{FixedNumber, Vec3},
};

pub mod line;
pub mod physics;
mod transform;
pub use transform::Transform;

pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fovy: FixedNumber,
    pub znear: FixedNumber,
    pub zfar: FixedNumber,
}

impl Camera {
    pub fn new(eye: Vec3, target: Vec3) -> Self {
        Self {
            eye,
            target,
            up: Vec3::unit_y(),
            fovy: 45.into(),
            znear: FixedNumber::fraction(10.into()),
            zfar: 1000.into(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PhysicBodies {
    Kinematic,
    Static,
    Rigidbody,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Aabb {
    // Local space min
    pub min: Vec3,
    // Local space max
    pub max: Vec3,
}

impl Aabb {
    pub fn new() -> Self {
        Self {
            min: Vec3::new(),
            max: Vec3::new(),
        }
    }

    pub fn colliding(
        &self,
        transform: &Transform,
        other: &Self,
        other_transform: &Transform,
    ) -> bool {
        //TODO: rotations
        let a_min = transform.position + self.min;
        let a_max = transform.position + self.max;
        let b_min = other_transform.position + other.min;
        let b_max = other_transform.position + other.max;

        return (a_min.x <= b_max.x && a_max.x >= b_min.x)
            && (a_min.y <= b_max.y && a_max.y >= b_min.y)
            && (a_min.z <= b_max.z && a_max.z >= b_min.z);
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Collision {
    pub other_entity: Entity,
    pub manifold: physics::Manifold,
}

pub struct CollisionList {
    collisions: [Option<Collision>; Self::MAX_COLLISIONS],
    next_id: usize,
}

impl CollisionList {
    const MAX_COLLISIONS: usize = 16;

    pub fn new() -> Self {
        Self {
            collisions: [None; Self::MAX_COLLISIONS],
            next_id: 0,
        }
    }

    pub fn add(&mut self, other_entity: Entity, manifold: physics::Manifold) {
        // TODO link up collision manifold + data
        self.collisions[self.next_id] = Some(Collision {
            other_entity,
            manifold,
        });

        self.next_id += 1;
        self.next_id = self.next_id % Self::MAX_COLLISIONS;
    }

    pub fn reset(&mut self) {
        for i in 0..Self::MAX_COLLISIONS {
            self.collisions[i] = None;
        }

        self.next_id = 0;
    }

    pub fn collisions(&self) -> Vec<Collision> {
        self.collisions
            .iter()
            .filter(|c| c.is_some())
            .map(|c| c.unwrap())
            .collect()
    }
}

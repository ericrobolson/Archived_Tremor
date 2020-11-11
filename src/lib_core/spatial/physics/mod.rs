use super::line::Line;
use super::*;

mod collision_detection;
use collision_detection::*;

mod capsule;
pub use capsule::Capsule;

mod sphere;
pub use sphere::Sphere;

pub trait CollisionShape: InternalCollisionShape {
    fn contains_point(&self, point: Vec3) -> bool;
    fn update_transform(&mut self, world_space_transform: Transform) {
        if self.previous_transform() != world_space_transform {
            self.update_world_space_transform(world_space_transform);
            self.set_previous_transform(world_space_transform);
        }
    }
}

pub trait InternalCollisionShape {
    fn update_world_space_transform(&mut self, world_space_transform: Transform);
    fn previous_transform(&self) -> Transform;
    fn set_previous_transform(&mut self, previous_world_space_transform: Transform);
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Manifold {
    pub penetration: FixedNumber,
    pub normal: Vec3,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CollisionShapes {
    Aabb(Aabb),
    Circle(Sphere),
    Capsule(Capsule),
}

impl CollisionShapes {
    pub fn default() -> Self {
        CollisionShapes::Circle(Sphere::default())
    }

    pub fn colliding(&self, other: &Self) -> Option<Manifold> {
        match self {
            CollisionShapes::Circle(sphere1) => match other {
                CollisionShapes::Circle(sphere2) => {
                    return circle_vs_circle(
                        sphere1.radius,
                        sphere1.world_space_transform(),
                        sphere2.radius,
                        sphere2.world_space_transform(),
                    );
                }
                CollisionShapes::Aabb(other_aabb) => {
                    return circle_vs_aabb();
                }
                CollisionShapes::Capsule(capsule) => {
                    return circle_vs_capsule(sphere1, capsule, true);
                }
            },
            CollisionShapes::Aabb(aabb) => match other {
                CollisionShapes::Aabb(other_aabb) => {
                    unimplemented!();
                    //return aabb_vs_aabb(aabb, transform, other_aabb, other_transform);
                }
                CollisionShapes::Circle(sphere) => {
                    return circle_vs_aabb();
                }
                CollisionShapes::Capsule(capsule) => {
                    return aabb_vs_capsule();
                }
            },
            CollisionShapes::Capsule(capsule) => match other {
                CollisionShapes::Capsule(other) => {
                    return capsule_vs_capsule(capsule, other);
                }
                CollisionShapes::Circle(sphere) => {
                    return circle_vs_capsule(sphere, capsule, false);
                }
                CollisionShapes::Aabb(aabb) => {
                    return aabb_vs_capsule();
                }
            },
        }
    }
}

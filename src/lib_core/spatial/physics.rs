use super::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Manifold {
    pub penetration: FixedNumber,
    pub normal: Vec3,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CollisionShapes {
    Aabb(Aabb),
    Circle { radius: FixedNumber },
    Capsule,
}

impl CollisionShapes {
    pub fn colliding(
        &self,
        transform: &Transform,
        other: &Self,
        other_transform: &Transform,
    ) -> Option<Manifold> {
        match self {
            CollisionShapes::Circle { radius } => match other {
                CollisionShapes::Circle {
                    radius: other_radius,
                } => {
                    return circle_vs_circle(*radius, transform, *other_radius, other_transform);
                }
                CollisionShapes::Aabb(other_aabb) => {
                    return circle_vs_aabb();
                }
                CollisionShapes::Capsule => {
                    return circle_vs_capsule();
                }
            },
            CollisionShapes::Aabb(aabb) => match other {
                CollisionShapes::Aabb(other_aabb) => {
                    return aabb_vs_aabb(aabb, transform, other_aabb, other_transform);
                }
                CollisionShapes::Circle {
                    radius: other_radius,
                } => {
                    return circle_vs_aabb();
                }
                CollisionShapes::Capsule => {
                    return aabb_vs_capsule();
                }
            },
            CollisionShapes::Capsule => match other {
                CollisionShapes::Capsule => {
                    return capsule_vs_capsule();
                }
                CollisionShapes::Circle {
                    radius: other_radius,
                } => {
                    return circle_vs_capsule();
                }
                CollisionShapes::Aabb(aabb) => {
                    return aabb_vs_capsule();
                }
            },
        }
    }
}

fn circle_vs_circle(
    a_radius: FixedNumber,
    a_transform: &Transform,
    b_radius: FixedNumber,
    b_transform: &Transform,
) -> Option<Manifold> {
    let normal = b_transform.position - a_transform.position;

    let r = a_radius + b_radius;
    let r_squared = r.sqrd();

    if normal.len_squared() > r_squared {
        return None;
    }

    let dist = normal.len();

    // Ensure circles aren't on the same position
    if dist != 0.into() {
        let penetration = r - dist;
        let normal = normal / dist;

        return Some(Manifold {
            penetration,
            normal,
        });
    } else {
        // Handle case where they're on the same position
        let penetration = a_radius;
        let normal = Vec3::unit_y();

        return Some(Manifold {
            penetration,
            normal,
        });
    }
}

fn circle_vs_aabb() -> Option<Manifold> {
    unimplemented!();
}

fn circle_vs_capsule() -> Option<Manifold> {
    unimplemented!();
}

fn aabb_vs_aabb(
    aabb1: &Aabb,
    transform: &Transform,
    other: &Aabb,
    other_transform: &Transform,
) -> Option<Manifold> {
    let a_min = transform.position + aabb1.min;
    let a_max = transform.position + aabb1.max;
    let b_min = other_transform.position + other.min;
    let b_max = other_transform.position + other.max;

    let normal = other_transform.position - transform.position;

    let a_extent = (a_max - a_min) / 2.into();
    let b_extent = (b_max - b_min) / 2.into();

    let combined_extent = a_extent + b_extent;

    // Uses separating axis theorem
    let x_overlap = combined_extent.x - normal.x.abs();
    let zero = FixedNumber::zero();
    if x_overlap > zero {
        let y_overlap = combined_extent.y - normal.y.abs();
        if y_overlap > zero {
            let z_overlap = combined_extent.z - normal.z.abs();

            if z_overlap > zero {
                // All axi overlap, meaning there's a collision

                // Penetration = greatest overlap
                let x_gt_y = x_overlap > y_overlap;
                let y_gt_z = y_overlap > z_overlap;
                let z_gt_x = z_overlap > x_overlap;

                let penetration = {
                    if x_gt_y && y_gt_z {
                        x_overlap
                    } else if !x_gt_y && y_gt_z {
                        y_overlap
                    } else {
                        z_overlap
                    }
                };

                // TODO: calculate normal

                return Some(Manifold {
                    penetration,
                    normal,
                });
            }
        }
    }

    /*
    let x_overlap = a_min.x <= b_max.x && a_max.x >= b_min.x;
    let y_overlap = a_min.y <= b_max.y && a_max.y >= b_min.y;
    let z_overlap = a_min.z <= b_max.z && a_max.z >= b_min.z;

    if !(x_overlap && y_overlap && z_overlap) {
        return None;
    }

    if aabb1.colliding(transform, other, other_transform) {
        return Some(Manifold {
            normal: Vec3::new(),
            penetration: 0.into(),
        });
    }

    // TODO: Replace with good version
    */

    None
}

fn aabb_vs_capsule() -> Option<Manifold> {
    unimplemented!();
}

fn capsule_vs_capsule() -> Option<Manifold> {
    unimplemented!();
}

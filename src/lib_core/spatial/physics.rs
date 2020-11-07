use super::line::Line;
use super::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Manifold {
    pub penetration: FixedNumber,
    pub normal: Vec3,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Capsule {
    pub radius: FixedNumber,
    pub length: FixedNumber,

    world_space_transform: Line,
}

impl Capsule {
    pub fn new(radius: FixedNumber, length: FixedNumber, world_transform: Transform) -> Self {
        let mut c = Self {
            radius,
            length,
            world_space_transform: Line::default(),
        };

        c.update_transform(world_transform);

        c
    }

    pub fn contains_point(&self, point: Vec3) -> bool {
        point_vs_capsule(point, &self)
    }

    pub fn update_transform(&mut self, world_space_transform: Transform) {
        self.world_space_transform = self.to_world_space(world_space_transform);
    }

    pub fn world_space_transform(&self) -> Line {
        self.world_space_transform
    }

    fn to_world_space(&self, transform: Transform) -> Line {
        // Create the center line is on the (x, 0, 0) plane
        let start: Vec3 = (0, 0, 0).into();

        let end = Vec3 {
            x: self.length,
            y: 0.into(),
            z: 0.into(),
        };

        // Offset it so the 'bottom' is against the origin planes.
        let start = start + self.radius.into();
        let end = end + self.radius.into();

        // TODO: Rotate

        // Add world position
        let start = start + transform.position;
        let end = end + transform.position;

        let line = Line { start, end };

        line
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Sphere {
    pub radius: FixedNumber,
    world_space_transform: Vec3,
}

impl Sphere {
    pub fn default() -> Self {
        Self::new(1.into(), Transform::default())
    }

    pub fn new(radius: FixedNumber, world_transform: Transform) -> Self {
        let mut c = Self {
            radius,
            world_space_transform: Vec3::new(),
        };

        c.update_transform(world_transform);

        c
    }

    pub fn contains_point(&self, point: Vec3) -> bool {
        point_in_sphere(point, self.world_space_transform, self.radius)
    }

    pub fn update_transform(&mut self, world_space_transform: Transform) {
        self.world_space_transform = self.to_world_space(world_space_transform);
    }

    pub fn world_space_transform(&self) -> Vec3 {
        self.world_space_transform
    }

    fn to_world_space(&self, transform: Transform) -> Vec3 {
        // Create the center line is on the (x, 0, 0) plane
        let p: Vec3 = (0, 0, 0).into();

        // Offset it so the 'bottom' is against the origin planes.
        let p = p + self.radius.into();

        // TODO: Rotate

        // Add world position
        p + transform.position
    }
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

    pub fn colliding(
        &self,
        transform: &Transform,
        other: &Self,
        other_transform: &Transform,
    ) -> Option<Manifold> {
        match self {
            CollisionShapes::Circle(sphere1) => match other {
                CollisionShapes::Circle(sphere2) => {
                    return circle_vs_circle(
                        sphere1.radius,
                        transform.position,
                        sphere2.radius,
                        other_transform.position,
                    );
                }
                CollisionShapes::Aabb(other_aabb) => {
                    return circle_vs_aabb();
                }
                CollisionShapes::Capsule(capsule) => {
                    return circle_vs_capsule();
                }
            },
            CollisionShapes::Aabb(aabb) => match other {
                CollisionShapes::Aabb(other_aabb) => {
                    return aabb_vs_aabb(aabb, transform, other_aabb, other_transform);
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
                    return capsule_vs_capsule(capsule, transform, other, other_transform);
                }
                CollisionShapes::Circle(sphere) => {
                    return circle_vs_capsule();
                }
                CollisionShapes::Aabb(aabb) => {
                    return aabb_vs_capsule();
                }
            },
        }
    }
}

fn circle_vs_circle_fast(
    a_radius: FixedNumber,
    a_position: Vec3,
    b_radius: FixedNumber,
    b_position: Vec3,
) -> bool {
    let normal = b_position - a_position;

    let r = a_radius + b_radius;
    let r_squared = r.sqrd();

    if normal.len_squared() > r_squared {
        return false;
    }

    return true;
}

fn circle_vs_circle(
    a_radius: FixedNumber,
    a_position: Vec3,
    b_radius: FixedNumber,
    b_position: Vec3,
) -> Option<Manifold> {
    let normal = b_position - a_position;

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
    // TODO: implement
    None
}

fn circle_vs_capsule() -> Option<Manifold> {
    // TODO: implement

    None
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
    //TODO:
    None
}

fn point_in_sphere(point: Vec3, sphere_pos: Vec3, radius: FixedNumber) -> bool {
    let r_sqrd = radius.sqrd();
    let dist = (point - sphere_pos).len_squared();

    dist <= r_sqrd
}

fn point_vs_capsule(point: Vec3, capsule: &Capsule) -> bool {
    let nearest_point = capsule.world_space_transform.closest_point(point);

    return point_in_sphere(point, nearest_point, capsule.radius);
}

fn capsule_vs_capsule(
    capsule1: &Capsule,
    capsule1_transform: &Transform,
    capsule2: &Capsule,
    capsule2_transform: &Transform,
) -> Option<Manifold> {
    // Capsule collisions: https://wickedengine.net/2020/04/26/capsule-collision-detection/

    // Get the two spheres for the points closest on the capsules
    // Capsule A
    // TODO: Move what you can from here to the 'capsule.to_world_space()' method to allow circle vs capsule collisions.
    // TODO: even add in a component that calculates this, instead of doing it for every collision? E.g. before doing collisions, you calculate all primitives and store them in their actual translated world position instead of local space
    /*
    let a_norm = c1_line.normalize();
    let a_line_end_offset = a_norm * c1.radius;
    let a_a = c1_line.end + a_line_end_offset; // TODO: Is this even necessary? may be able to get rid of the norm if so
    let a_b = c1_line.start - a_line_end_offset;
    */
    let a_a = capsule1.world_space_transform.end;
    let a_b = capsule1.world_space_transform.start;

    // Capsule B
    // TODO: Move what you can from here to the 'capsule.to_world_space()' method to allow circle vs capsule collisions.
    // TODO: even add in a component that calculates this, instead of doing it for every collision? E.g. before doing collisions, you calculate all primitives and store them in their actual translated world position instead of local space
    /*
    let b_norm = c2_line.normalize();
    let b_line_end_offset = b_norm * c2.radius;
    let b_a = c2_line.end + b_line_end_offset;
    let b_b = c2_line.start - b_line_end_offset;
    */
    let b_a = capsule2.world_space_transform.end;
    let b_b = capsule2.world_space_transform.start;

    // Vectors between line endpoints
    let v0 = b_a - a_a;
    let v1 = b_b - a_a;
    let v2 = b_a - a_b;
    let v3 = b_b - a_b;

    // Squared distances
    let d0 = v0.dot(v0);
    let d1 = v1.dot(v1);
    let d2 = v2.dot(v2);
    let d3 = v3.dot(v3);

    // Select best potential endpoint on Capsule A
    let best_a = {
        if d2 < d0 || d2 < d1 || d3 < d0 || d3 < d1 {
            a_a
        } else {
            a_b
        }
    };

    // Select point on capsule B line segment nearest to best potential endpoint on A capsule
    let best_b = Line {
        start: b_a,
        end: b_b,
    }
    .closest_point(best_a);

    // Do the same for capsule A line segment
    let best_a = Line {
        start: a_a,
        end: a_b,
    }
    .closest_point(best_b);

    return circle_vs_circle(capsule1.radius, best_a, capsule2.radius, best_b);
}

use super::*;

pub fn circle_vs_circle(
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

pub fn circle_vs_aabb() -> Option<Manifold> {
    // TODO: implement
    None
}

pub fn circle_vs_capsule(
    sphere: &Sphere,
    capsule: &Capsule,
    circle_is_first_entity: bool,
) -> Option<Manifold> {
    let nearest_point = capsule
        .world_space_transform()
        .closest_point(sphere.world_space_transform());

    let collision = circle_vs_circle(
        sphere.radius,
        sphere.world_space_transform(),
        capsule.radius,
        nearest_point,
    );

    if collision.is_some() && !circle_is_first_entity {
        // Not exactly sure why this occurs, but when there's a collision, the penetration has the wrong sign. I suspect it has to do with the order the capsule sphere and sphere are processed.
        let mut updating_pen = collision.unwrap();

        updating_pen.penetration *= (-1).into();
        return Some(updating_pen);
    }

    collision
}

pub fn aabb_vs_aabb(
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

pub fn aabb_vs_capsule() -> Option<Manifold> {
    //TODO:
    None
}

pub fn point_in_sphere(point: Vec3, sphere_pos: Vec3, radius: FixedNumber) -> bool {
    let r_sqrd = radius.sqrd();
    let dist = (point - sphere_pos).len_squared();

    dist <= r_sqrd
}

pub fn point_vs_capsule(point: Vec3, capsule: &Capsule) -> bool {
    let nearest_point = capsule.world_space_transform().closest_point(point);

    return point_in_sphere(point, nearest_point, capsule.radius);
}

pub fn capsule_vs_capsule(capsule1: &Capsule, capsule2: &Capsule) -> Option<Manifold> {
    // Capsule collisions: https://wickedengine.net/2020/04/26/capsule-collision-detection/

    // Get the two spheres for the points closest on the capsules
    // Capsule A
    let a_transform = capsule1.world_space_transform();
    let a_a = a_transform.end;
    let a_b = a_transform.start;

    // Capsule B
    let b_transform = capsule2.world_space_transform();

    let b_a = b_transform.end;
    let b_b = b_transform.start;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collision_circle_vs_capsule_returns_some() {
        let sphere = Sphere::new(10.into(), Transform::default());
        let capsule = Capsule::new(10.into(), 10.into(), Transform::default());

        let a = CollisionShapes::Circle(sphere);
        let b = CollisionShapes::Capsule(capsule);

        let collision = a.colliding(&b);
        assert_eq!(true, collision.is_some());
    }
}

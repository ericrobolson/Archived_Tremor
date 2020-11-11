use super::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Sphere {
    pub radius: FixedNumber,
    world_space_transform: Vec3,
    previous_transform: Transform,
}

impl Sphere {
    pub fn default() -> Self {
        Self::new(1.into(), Transform::default())
    }

    pub fn new(radius: FixedNumber, world_transform: Transform) -> Self {
        let mut c = Self {
            radius,
            world_space_transform: Vec3::new(),
            previous_transform: world_transform,
        };

        c.world_space_transform = c.to_world_space(world_transform);

        c
    }

    pub fn world_space_transform(&self) -> Vec3 {
        self.world_space_transform
    }

    fn to_world_space(&self, transform: Transform) -> Vec3 {
        // Create the center line is on the (x, 0, 0) plane
        let p: Vec3 = (0, 0, 0).into();

        // Offset it so the 'bottom' is against the origin planes.
        let p = p + self.radius.into();

        // Rotate
        let p = transform.rotation.rotate_vec3(p);

        // Add world position
        p + transform.position
    }
}

impl InternalCollisionShape for Sphere {
    fn update_world_space_transform(&mut self, world_space_transform: Transform) {
        self.world_space_transform = self.to_world_space(world_space_transform);
    }
    fn previous_transform(&self) -> Transform {
        self.previous_transform
    }
    fn set_previous_transform(&mut self, previous_world_space_transform: Transform) {
        self.previous_transform = previous_world_space_transform;
    }
}

impl CollisionShape for Sphere {
    fn contains_point(&self, point: Vec3) -> bool {
        point_in_sphere(point, self.world_space_transform, self.radius)
    }
}

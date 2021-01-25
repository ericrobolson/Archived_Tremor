use super::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Capsule {
    pub radius: FixedNumber,
    pub length: FixedNumber,

    world_space_transform: Line,
    previous_transform: Transform,
}

impl Capsule {
    pub fn new(radius: FixedNumber, length: FixedNumber, world_transform: Transform) -> Self {
        let mut c = Self {
            radius,
            length,
            world_space_transform: Line::default(),
            previous_transform: world_transform,
        };

        c.world_space_transform = c.to_world_space(world_transform);

        c
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

        // Rotate
        let start = transform.rotation.rotate_vec3(start);
        let end = transform.rotation.rotate_vec3(end);

        // Add world position
        let start = start + transform.position;
        let end = end + transform.position;

        let line = Line { start, end };

        line
    }
}

impl InternalCollisionShape for Capsule {
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

impl CollisionShape for Capsule {
    fn contains_point(&self, point: Vec3) -> bool {
        point_vs_capsule(point, &self)
    }
}

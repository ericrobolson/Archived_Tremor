use game_math::f32::*;

#[derive(Copy, Clone)]
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
    pub valid: bool,
}

impl BoundingBox {
    pub fn default() -> Self {
        Self::new(Vec3::default(), Vec3::default())
    }

    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self {
            min,
            max,
            valid: false,
        }
    }

    pub fn get_aabb(&self, m: Mat4) -> Self {
        let mut min = m[3].vec3();
        let mut max = min;

        let right = m[0].vec3();
        let v0 = right * self.min.x;
        let v1 = right * self.max.x;
        min += v0.min(v1);
        max += v0.max(v1);

        let up = m[1].vec3();
        let v0 = up * self.min.y;
        let v1 = up * self.max.y;
        min += v0.min(v1);
        max += v0.max(v1);

        let back = m[2].vec3();
        let v0 = back * self.min.z;
        let v1 = back * self.max.z;
        min += v0.min(v1);
        max += v0.max(v1);

        Self::new(min, max)
    }
}

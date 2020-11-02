use cgmath::{perspective, Deg, Matrix4, Perspective, Point3, Vector3};

use crate::lib_core::{
    ecs::World,
    math::{FixedNumber, Vec3},
    spatial,
};

impl Into<Point3<f32>> for Vec3 {
    fn into(self) -> Point3<f32> {
        let p: (f32, f32, f32) = (self.x.into(), self.y.into(), self.z.into());
        p.into()
    }
}

impl Into<Vector3<f32>> for Vec3 {
    fn into(self) -> Vector3<f32> {
        let p: (f32, f32, f32) = (self.x.into(), self.y.into(), self.z.into());
        p.into()
    }
}

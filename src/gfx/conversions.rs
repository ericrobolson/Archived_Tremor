use cgmath::{perspective, Deg, Matrix4, Perspective, Point3, Vector3};

use crate::lib_core::{
    ecs::World,
    math::{FixedNumber, Quaternion, Vec3},
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

impl Into<cgmath::Matrix4<f32>> for Quaternion {
    fn into(self) -> cgmath::Matrix4<f32> {
        let (w, x, y, z) = self.wxyz();
        let q = cgmath::Quaternion::new(w.into(), x.into(), y.into(), z.into());
        q.into()
    }
}

fn to_rad(f: FixedNumber) -> cgmath::Rad<f32> {
    cgmath::Rad(f.into())
}

fn rot_matrix(q: Quaternion) -> cgmath::Matrix4<f32> {
    q.into()
}

impl Into<cgmath::Matrix4<f32>> for spatial::Transform {
    fn into(self) -> Matrix4<f32> {
        // Scale
        let scale_mat = Matrix4::<f32>::from_nonuniform_scale(
            self.scale.x.into(),
            self.scale.y.into(),
            self.scale.z.into(),
        );
        // Rotate
        let rot_mat = rot_matrix(self.rotation);

        // Position?
        let pos_mat = Matrix4::from_translation(self.position.into());
        return pos_mat * rot_mat * scale_mat;
    }
}

pub fn transformation_matrix(transform: spatial::Transform) -> cgmath::Matrix4<f32> {
    transform.into()
}

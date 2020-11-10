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

fn to_rad(f: FixedNumber) -> cgmath::Rad<f32> {
    cgmath::Rad(f.into())
}

fn rot_matrix(v: Vec3) -> cgmath::Matrix4<f32> {
    let angle_x = Matrix4::<f32>::from_angle_x(to_rad(v.x));
    let angle_y = Matrix4::<f32>::from_angle_y(to_rad(v.y));
    let angle_z = Matrix4::<f32>::from_angle_z(to_rad(v.z));

    // TODO: instead, pull in the quaternion

    angle_x * angle_y * angle_z
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

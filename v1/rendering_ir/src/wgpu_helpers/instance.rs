use cgmath::Rotation3;

/// The representation of an instance for a model.
#[derive(Copy, Clone)]
pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub scale: cgmath::Vector3<f32>,
}

impl Instance {
    pub fn default() -> Self {
        Self::new(
            [0.0, 0.0, 0.0].into(),
            cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0)),
            [1.0, 1.0, 1.0].into(),
        )
    }

    pub fn new(
        position: cgmath::Vector3<f32>,
        rotation: cgmath::Quaternion<f32>,
        scale: cgmath::Vector3<f32>,
    ) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }
}

/// What actually gets uploaded to the GPU
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    pub model_transform: [[f32; 4]; 4],
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model_transform: (cgmath::Matrix4::<f32>::from_translation(self.position)
                * cgmath::Matrix4::<f32>::from(self.rotation)
                * cgmath::Matrix4::<f32>::from_nonuniform_scale(
                    self.scale.x,
                    self.scale.y,
                    self.scale.z,
                ))
            .into(),
        }
    }
}

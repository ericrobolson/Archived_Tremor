use rendering_ir::{camera3d::Camera3d, wgpu_helpers::OPENGL_TO_WGPU_MATRIX};
use std::convert::TryInto;

use crate::gfx::uniform_container::{ToGpuRaw, UniformContainer};
use game_math::f32::*;

pub const MAX_NUM_JOINTS: usize = 128; // when updating this, ensure shaders are updated.

pub use material_uniform::{MaterialUbo, MaterialUniformContainer};
pub use model_uniform::{ModelUbo, ModelUniformContainer};
pub use node_uniform::{NodeUbo, NodeUniformContainer};
pub use scene_uniform::{SceneUbo, SceneUniformContainer};

mod scene_uniform {
    use super::*;
    pub type SceneUniformContainer = UniformContainer<SceneUbo, SceneUboRaw>;

    pub struct SceneUbo {
        projection: Mat4,
        view: Mat4,
        view_projection: Mat4,
        cam_pos: Vec3,
    }

    impl SceneUbo {
        pub fn new(camera: &Camera3d) -> Self {
            let mut scene = Self {
                view_projection: Mat4::default(),
                projection: Mat4::default(),
                view: Mat4::default(),
                cam_pos: Vec3::default(),
            };

            scene.update(camera);

            scene
        }

        pub fn update(&mut self, camera: &Camera3d) {
            let proj = camera.projection_matrix();

            let project = Mat4::from_raw(camera.projection_matrix().into());
            let view = Mat4::from_raw(camera.view_matrix().into());
            let cam_pos = Vec3::from_raw(camera.eye());

            self.projection = project;
            self.view = view;
            self.cam_pos = cam_pos;

            let viewproj = {
                let proj = camera.projection_matrix();
                let view = camera.view_matrix();
                OPENGL_TO_WGPU_MATRIX * proj * view
            };

            // WGPU has a different matrix, so convert it here.
            self.view_projection = Mat4::from_raw(viewproj.into());
        }
    }

    impl ToGpuRaw<SceneUboRaw> for SceneUbo {
        fn to_gpu_raw(&self) -> SceneUboRaw
        where
            SceneUboRaw: bytemuck::Pod + bytemuck::Zeroable,
        {
            SceneUboRaw {
                view_projection: self.view_projection.to_raw(),
                projection: self.projection.to_raw(),
                view: self.view.to_raw(),
                cam_position: self.cam_pos.to_raw(),
            }
        }
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
    pub struct SceneUboRaw {
        view_projection: RawMat4,
        projection: RawMat4,
        view: RawMat4,
        cam_position: RawVec3,
    }
}

mod node_uniform {
    use super::*;
    pub type NodeUniformContainer = UniformContainer<NodeUbo, NodeUboRaw>;

    pub struct NodeUbo {
        pub matrix: Mat4,
        pub joint_matrix: [Mat4; MAX_NUM_JOINTS],
        pub joint_count: f32,
    }

    impl ToGpuRaw<NodeUboRaw> for NodeUbo {
        fn to_gpu_raw(&self) -> NodeUboRaw
        where
            NodeUboRaw: bytemuck::Pod + bytemuck::Zeroable,
        {
            let joints: Vec<RawMat4> = self.joint_matrix.iter().map(|m| m.to_raw()).collect();

            NodeUboRaw {
                matrix: self.matrix.to_raw(),
                joint_matrix: joints.try_into().unwrap(),
                joint_count: self.joint_count,
            }
        }
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]

    pub struct NodeUboRaw {
        matrix: RawMat4,
        joint_matrix: [RawMat4; super::MAX_NUM_JOINTS],
        joint_count: f32,
    }
}

type RawMat4 = [[f32; 4]; 4];
type RawVec3 = [f32; 3];

mod model_uniform {
    use super::*;
    pub type ModelUniformContainer = UniformContainer<ModelUbo, ModelUboRaw>;

    pub struct ModelUbo {
        pub model: Mat4,
    }

    impl ToGpuRaw<ModelUboRaw> for ModelUbo {
        fn to_gpu_raw(&self) -> ModelUboRaw
        where
            ModelUboRaw: bytemuck::Pod + bytemuck::Zeroable,
        {
            ModelUboRaw {
                model: self.model.to_raw(),
            }
        }
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
    pub struct ModelUboRaw {
        model: RawMat4,
    }
}

mod material_uniform {
    use super::*;
    pub type MaterialUniformContainer = UniformContainer<MaterialUbo, MaterialUboRaw>;

    pub struct MaterialUbo {
        /// The base color of the material
        pub base_color_factor: [f32; 4],
        /// The metallic factor of the material. 0..1
        pub metallic_factor: f32,
        /// The roughness factor of the material. 0..1
        pub roughness_factor: f32,
        /// The emissive factor of the material. 0..1
        pub emissive_factor: [f32; 3],

        pub metallic_roughness_texture_factor: f32,
        pub base_color_texture_factor: f32,
        pub normal_texture_factor: f32,
        pub occlusion_texture_factor: f32,
        pub emissive_texture_factor: f32,
    }

    impl ToGpuRaw<MaterialUboRaw> for MaterialUbo {
        fn to_gpu_raw(&self) -> MaterialUboRaw
        where
            MaterialUboRaw: bytemuck::Pod + bytemuck::Zeroable,
        {
            MaterialUboRaw {
                base_color_factor: self.base_color_factor,
                metallic_factor: self.metallic_factor,
                roughness_factor: self.roughness_factor,
                emissive_factor: self.emissive_factor,

                metallic_roughness_texture_factor: self.metallic_roughness_texture_factor,
                base_color_texture_factor: self.base_color_texture_factor,
                normal_texture_factor: self.normal_texture_factor,
                occlusion_texture_factor: self.occlusion_texture_factor,
                emissive_texture_factor: self.emissive_texture_factor,
            }
        }
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
    pub struct MaterialUboRaw {
        pub base_color_factor: [f32; 4],
        pub metallic_factor: f32,
        pub roughness_factor: f32,
        pub emissive_factor: [f32; 3],

        pub metallic_roughness_texture_factor: f32,
        pub base_color_texture_factor: f32,
        pub normal_texture_factor: f32,
        pub occlusion_texture_factor: f32,
        pub emissive_texture_factor: f32,
    }
}

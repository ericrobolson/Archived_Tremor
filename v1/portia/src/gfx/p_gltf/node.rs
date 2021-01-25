use super::{mesh::Mesh, skin::Skin, SmartPointer, MAX_NUM_JOINTS};
use crate::gfx::{
    uniforms::{NodeUbo, NodeUniformContainer},
    DeviceQueue,
};
use data_structures::Id;
use game_math::f32::*;
pub struct Node {
    pub parent: Option<Id>,
    pub index: usize,
    pub children: Vec<Node>,
    pub matrix: Mat4,
    pub name: String,
    pub mesh: Option<Mesh>,
    pub skin: Option<Skin>,
    pub translation: Vec3,
    pub scale: Vec3,
    pub rotation: Quaternion,
    pub node_ubo: NodeUniformContainer,
}
impl Node {
    pub fn local_matrix(&self) -> Mat4 {
        // TODO: how to cache this?
        Mat4::translate(Mat4::i32(1), self.translation)
            * self.rotation.to_mat4()
            * Mat4::i32(1).scale(self.scale)
            * self.matrix
    }

    pub fn get_matrix(&self) -> Mat4 {
        unimplemented!();
        /*
        // TODO: how to cache this?
        let mut m = self.local_matrix();

        if let Some(parent) = &self.parent {
            m = parent.get_matrix() * m;
        }

        m
        */
    }

    /// Write data to GPU
    pub fn buffer(&mut self) {
        if let Some(ref mut mesh) = self.mesh {
            mesh.buffer();
        }

        for child in self.children.iter_mut() {
            child.buffer();
        }
    }

    pub fn mesh(&self) -> Option<&Mesh> {
        self.mesh.as_ref()
    }

    pub fn update(&mut self) {
        return;
        /*
        //https://github.com/SaschaWillems/Vulkan-glTF-PBR/blob/master/base/VulkanglTFModel.hpp#L521
        let m = {
            if self.mesh.is_some() {
                self.get_matrix()
            } else {
                Mat4::default()
            }
        };

        if let Some(ref mut mesh) = self.mesh {
            mesh.uniform_block.set_matrix(m);

            // Update joint matrices if they exist
            if let Some(ref mut skin) = self.skin {
                let inverse_transform = m.inverse();
                let num_joints = { skin.joints.len().min(MAX_NUM_JOINTS) };
                for i in 0..num_joints {
                    let joint_node = &skin.joints[i];
                    let joint_matrix = joint_node.get_matrix() * skin.inverse_bind_matrices[i];
                    let joint_matrix = inverse_transform * joint_matrix;
                    mesh.uniform_block.set_joint_matrix(i, joint_matrix);
                }

                mesh.uniform_block.set_joint_count(num_joints as f32);
            }
        }

        for child in self.children.iter_mut() {
            child.update();
        }
        */
    }
}

use game_math::f32::*;

use super::{
    animations::Animation,
    boundingbox::BoundingBox,
    material::Material,
    node::Node,
    primitive::Primitive,
    skin::Skin,
    textures::{Texture, TextureSampler},
    vert_indices::{Indices, Vertex},
    SmartPointer, MAX_NUM_JOINTS,
};

pub struct Mesh {
    uniform_buffer: UniformBuffer,
    pub uniform_block: UniformBlock,
    pub primitives: Vec<Primitive>,
    bounding_box: BoundingBox,
    pub index_start: u32,
    pub index_len: u32,
    pub base_vertex: u32,
}
impl Mesh {
    pub fn new(
        primitives: Vec<Primitive>,
        matrix: Mat4,
        index_start: u32,
        index_len: u32,
        base_vertex: u32,
    ) -> Self {
        let mut bounding_box = BoundingBox::default();
        for primitive in &primitives {
            if primitive.bounding_box.valid && !bounding_box.valid {
                bounding_box = primitive.bounding_box;
                bounding_box.valid = true;
            }

            bounding_box.min = bounding_box.min.min(primitive.bounding_box.min);
            bounding_box.max = bounding_box.max.max(primitive.bounding_box.max);
        }

        Self {
            index_start,
            index_len,
            base_vertex,
            primitives,
            bounding_box,
            uniform_buffer: UniformBuffer {},
            uniform_block: UniformBlock {
                matrix,
                joint_count: 0.,
                joint_matrix: [Mat4::default(); MAX_NUM_JOINTS],
                dirty_matrix: true, // True to force a buffer
                dirty_joints: true, // True to force a buffer
            },
        }
    }

    fn set_primitives(&mut self, primitives: Vec<Primitive>) {
        self.primitives = primitives;
    }

    /// Buffer data to the GPU.
    pub fn buffer(&mut self) {
        self.uniform_block.buffer();
    }
}

pub struct UniformBuffer {}

/// Uniforms block. Not directly uploaded to GPU, but does upload a small subset of its data.
pub struct UniformBlock {
    matrix: Mat4,
    joint_matrix: [Mat4; MAX_NUM_JOINTS],
    joint_count: f32,
    dirty_matrix: bool,
    dirty_joints: bool,
}

impl UniformBlock {
    fn clear_dirty(&mut self) {
        self.dirty_matrix = false;
        self.dirty_joints = false;
    }

    fn dirty(&self) -> bool {
        self.dirty_matrix || self.dirty_joints
    }

    pub fn set_matrix(&mut self, m: Mat4) {
        self.dirty_matrix = true;
        self.matrix = m;
    }

    pub fn set_joint_matrix(&mut self, i: usize, m: Mat4) {
        self.dirty_joints = true;
        self.joint_matrix[i] = m;
    }

    pub fn set_joint_count(&mut self, count: f32) {
        self.dirty_joints = true;
        self.joint_count = count;
    }

    /// Buffers the data to the GPU
    pub fn buffer(&mut self) {
        if self.dirty() {
            // TODO: if only a matrix was buffered, just write that. If everything was buffered, write that. Try to make it so it only uploads data that was changed.
            // TODO: actually buffer necessary data

            if self.dirty_matrix && !self.dirty_joints {
                // TODO: only buffer matrix
            } else {
                // TODO: buffer everything.
                let raw = self.into_raw();
            }

            self.clear_dirty();
            unimplemented!("implement buffering");
        }
    }

    /// Converts the UniformBlock into the raw representation for the GPU.
    fn into_raw(&self) -> UniformBlockRaw {
        unimplemented!();
    }
}

type RawMat4 = [[f32; 4]; 4];

#[repr(C)] // We need this for Rust to store our data correctly for the shaders
#[derive(Debug, Copy, Clone)] // This is so we can store this in a buffer
struct UniformBlockRaw {
    matrix: RawMat4,
    joint_matrix: [RawMat4; MAX_NUM_JOINTS],
    joint_count: f32,
}

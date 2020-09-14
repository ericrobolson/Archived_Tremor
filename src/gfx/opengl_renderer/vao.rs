extern crate gl;

use super::{vbo::*, vertices::*};

pub struct Vao {
    id: gl::types::GLuint,
    indices_to_render: Option<usize>,
}

impl Vao {
    /// Create a new VAO
    pub fn new() -> Self {
        let mut vao: gl::types::GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
        }

        Self {
            id: vao,
            indices_to_render: None,
        }
    }

    /// Given a VBO and some vertices, buffer them
    pub fn buffer(&mut self, vbo: &mut Vbo, vertices: &Vertices) {
        vbo.buffer(vertices);
        vbo.bind();
        self.bind();

        // Here we build up the VAO with all proper offsets
        for (i, vert_data_component_len) in vertices.components_per_vertex().iter().enumerate() {
            unsafe {
                let prev_offset = {
                    if i == 0 {
                        std::ptr::null()
                    } else {
                        let mut offset = 0;
                        //sum all previous ones
                        for j in 0..i {
                            offset += vertices.components_per_vertex()[j];
                        }

                        (offset * std::mem::size_of::<f32>()) as *const gl::types::GLvoid
                    }
                };

                gl::EnableVertexAttribArray(i as u32);
                gl::VertexAttribPointer(
                    i as u32, // index of the generic vertex attribute ("layout (location = 0)")
                    *vert_data_component_len as i32, // the number of components per generic vertex attribute
                    gl::FLOAT,                       // data type
                    gl::FALSE,                       // normalized (int-to-float conversion)
                    (vertices.stride_length() * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
                    prev_offset, // offset of the first component
                );
            }
        }

        self.indices_to_render = Some(vertices.index_length());
        self.unbind();

        vbo.unbind();
    }

    /// Render the VAO
    pub fn render(&self, vbo: &Vbo) {
        if self.indices_to_render.is_none() {
            return;
        }

        self.bind();
        vbo.bind();

        unsafe {
            gl::DrawArrays(
                gl::TRIANGLES,
                0,                                      // starting index in the enabled arrays
                self.indices_to_render.unwrap() as i32, // number of indices to be rendered
            )
        }

        vbo.unbind();
        self.unbind();
    }

    /// Bind the VAO
    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }

    /// Unbind the VAO
    pub fn unbind(&self) {
        unsafe {
            gl::BindVertexArray(0);
        }
    }
}

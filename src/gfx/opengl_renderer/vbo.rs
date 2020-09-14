extern crate gl;

use super::vertices::*;

pub struct Vbo {
    id: gl::types::GLuint,
}

impl Vbo {
    /// Create a new VBO
    pub fn new() -> Self {
        let mut vbo: gl::types::GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut vbo);
        }

        Self { id: vbo }
    }

    /// Buffer the vertices to the VBO
    pub fn buffer(&mut self, vertices: &Vertices) {
        self.bind();

        let vertices = vertices.vertices();

        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                vertices.as_ptr() as *const gl::types::GLvoid,
                gl::STATIC_DRAW,
            );
        }

        self.unbind();
    }

    /// Bind the VBO
    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.id);
        }
    }

    /// Unbind the VBO
    pub fn unbind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }
}

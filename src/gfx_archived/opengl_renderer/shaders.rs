use std::ffi::{CStr, CString};

extern crate gl;

use super::helpers;
use helpers::*;

/// Object that represents a OpenGL shader
pub struct Shader {
    /// The GL id of the shader
    id: gl::types::GLuint,
}

impl Shader {
    /// Create a new shader from a vert source
    pub fn from_vert_source(source: &CStr) -> Result<Self, String> {
        Self::from_source(source, gl::VERTEX_SHADER)
    }

    /// Create a new shader from a fragment source
    pub fn from_frag_source(source: &CStr) -> Result<Self, String> {
        Self::from_source(source, gl::FRAGMENT_SHADER)
    }

    /// Initialize the shader
    fn from_source(source: &CStr, kind: gl::types::GLenum) -> Result<Self, String> {
        let id = shader_from_source(source, kind)?;
        Ok(Self { id: id })
    }

    /// Returns the id of the compiled shader
    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

/// Initialize the OpenGL shader
fn shader_from_source(source: &CStr, kind: gl::types::GLuint) -> Result<gl::types::GLuint, String> {
    let id = unsafe { gl::CreateShader(kind) };

    unsafe {
        gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
        gl::CompileShader(id);
    }

    // Error handling
    let mut success: gl::types::GLint = 1;
    unsafe {
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
    }

    if success == 0 {
        let mut len: gl::types::GLint = 0;
        unsafe {
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
        }
        let error: CString = create_whitespace_cstring_with_len(len as usize);

        unsafe {
            gl::GetShaderInfoLog(
                id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut gl::types::GLchar,
            );
        }

        return Err(error.to_string_lossy().into_owned());
    }

    return Ok(id);
}

//use crate::ecs::World;

use std::ffi::CStr;

use glutin::{self, PossiblyCurrent};

extern crate gl;

mod helpers;

mod debug_primitives;
mod program;
mod shaders;
mod sprites;
mod texture;
mod vao;
mod vbo;
mod vertices;

#[derive(Copy, Clone)]
pub struct Resolution {
    pub height: u32,
    pub width: u32,
}

pub struct OpenGlRenderer {
    sprite_pass: sprites::SpritePass,
    debug_pass: debug_primitives::DebugPass,
    pub resolution: Resolution,
}

impl OpenGlRenderer {
    /// Create a new OpenGL render backend
    pub fn new(gl_context: &glutin::Context<PossiblyCurrent>, height: u32, width: u32) -> Self {
        let resolution = Resolution {
            width: width,
            height: height,
        };

        gl::load_with(|ptr| gl_context.get_proc_address(ptr) as *const _);

        let version = unsafe {
            let data = CStr::from_ptr(gl::GetString(gl::VERSION) as *const _)
                .to_bytes()
                .to_vec();
            String::from_utf8(data).unwrap()
        };

        let sprite_pass = sprites::SpritePass::new();
        let debug_pass = debug_primitives::DebugPass::new();

        Self {
            sprite_pass: sprite_pass,
            resolution: resolution,
            debug_pass: debug_pass,
        }
    }

    /// Set the viewport
    pub fn set_resolution(&mut self, width: i32, height: i32) {
        self.resolution = Resolution {
            width: width as u32,
            height: height as u32,
        };

        unsafe {
            gl::Viewport(0, 0, width, height);
        }
    }

    /// Execute all render passes
    pub fn render(&mut self) {
        // Clear the canvas
        unsafe {
            let color = 212.0 / 255.0;
            let color = 0.0;

            gl::ClearColor(color, color, color, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        // Execute the various passes
        self.sprite_pass.render(self.resolution);
        self.debug_pass.render(self.resolution);
    }
}

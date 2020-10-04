mod opengl_renderer;
pub use opengl_renderer::OpenGlRenderer;

pub struct GfxVm {}

impl GfxVm {
    pub fn new() -> Self {
        Self {}
    }
}

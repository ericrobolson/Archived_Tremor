fn main() {
    rendering_ir::build::gfx_shader_build().unwrap();
    rendering_ir::build::copy_res().unwrap();
}

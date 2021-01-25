#version 450

layout(location=0) in vec2 a_pos;
layout(location=1) in vec2 a_tex_coords;

layout(location=0) out vec2 tex_coords;

void main() {
    tex_coords = a_tex_coords;
    gl_Position =  vec4(a_pos, 0.0, 1.0);
}
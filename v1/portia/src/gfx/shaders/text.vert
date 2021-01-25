#version 450

layout(location=0) in vec2 a_pos;
layout(location=1) in vec2 a_tex_coords;
layout(location=2) in vec4 a_text_color;

layout(location=0) out vec2 TexCoords;
layout(location=1) out vec4 text_color;


void main() {
    TexCoords = a_tex_coords;
    //gl_Position = u_view_proj * vec4(a_pos, 0.0, 1.0);
    text_color = a_text_color;
    gl_Position =  vec4(a_pos, 0.0, 1.0);
}
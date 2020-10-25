#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_col;

layout(location=0) out vec3 v_color;


layout(set=0, binding=0)
uniform Uniforms{
    vec3 u_view_position;
    mat4 u_view_proj;
    vec2 u_viewport_size;
};

void main() {
    v_color = a_col;
    gl_Position = u_view_proj * vec4(a_position, 1.0);
}
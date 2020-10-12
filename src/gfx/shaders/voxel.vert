#version 450

layout(location=0) in vec3 a_position;

layout(set=0, binding=0)
uniform Uniforms{
    vec3 u_view_position;
    mat4 u_view_proj;
    vec2 u_viewport_size;
};


void main() {
    vec4 model_space = vec4(a_position, 1.0);

    gl_Position = u_view_proj * model_space;
}
#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in uint a_palette_index;

layout(location=0) out uint palette_index;


layout(set=0, binding=0)
uniform Uniforms{
    vec3 u_view_position;
    mat4 u_view_proj;
    vec2 u_viewport_size;
};

// layout1 = palette texture

layout(set=2, binding=0)
uniform ModelTransform{
    mat4 model_transform;
};

void main() {
    palette_index = a_palette_index;
    gl_Position = u_view_proj * model_transform * vec4(a_position, 1.0);
}
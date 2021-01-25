#version 450

// Inputs
layout (location = 0) in vec3 in_world_pos;
layout (location = 1) in vec3 in_normal;
layout (location = 2) in vec2 in_uv0;
layout (location = 3) in vec2 in_uv1;

// Uniforms
layout (set = 3, binding = 0) uniform MaterialUbo {
	vec4 base_color_factor;
    float metallic_factor;
    float roughness_factor;
    vec3 emissive_factor;


    // TODO: when linking these up, multiply by the texture. This ensures consistent behavior even if no texture is specified.
    float metallic_roughness_texture_factor;
    float base_color_texture_factor;
    float normal_texture_factor;
    float occlusion_texture_factor;
    float emissive_texture_factor;
} material;

// Outputs
layout(location=0) out vec4 f_color;

void main() {
    vec4 tex01_color = vec4(1.0, 0.0, 0.0, 1.0) * material.base_color_texture_factor;
    f_color = material.base_color_factor;
    f_color = vec4(1.0, 1.0, 0.0, 1.0);
}
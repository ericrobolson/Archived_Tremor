#version 450
#extension GL_EXT_samplerless_texture_functions: require

layout(location=0) flat in uint palette_index;
layout(location=0) out vec4 f_color;

layout(set=1, binding=0) uniform texture1D palette_tex;
layout(set=1, binding=1) uniform sampler1D palette_sampler;

void main() {
    f_color = texelFetch(palette_tex, int(palette_index), 0);
}
#version 450

layout(location=0) in vec2 tex_coords;
layout(location=0) out vec4 color;

layout(set=0, binding=0) uniform texture2D t_img;
layout(set=0, binding=1) uniform sampler s_img;

void main()
{    
    vec4 sampled = texture(sampler2D(t_img, s_img), tex_coords);
    color = sampled;
}  

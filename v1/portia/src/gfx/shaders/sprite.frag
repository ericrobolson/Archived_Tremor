#version 450

const float MIN_ALPHA = 0.1;


layout(location=0) in vec2 TexCoords;
layout(location=1) in vec4 text_color;
layout(location=0) out vec4 color;

layout(set=0, binding=0) uniform texture2D t_character;
layout(set=0, binding=1) uniform sampler s_character;

void main()
{    
    vec4 sampled = texture(sampler2D(t_character, s_character), TexCoords);
    if (sampled.a < MIN_ALPHA){
        discard;
    }

    color = text_color * sampled;
}  

#version 450

layout(location=0) out vec4 f_color;


layout(set=0, binding=0)
uniform Uniforms{
    vec3 u_view_position;
    mat4 u_view_proj;
    vec2 u_viewport_size;
};


void main(){
    vec3 col = vec3(0.4,0.5,0.1);

    f_color = vec4(col, 1.0);
}
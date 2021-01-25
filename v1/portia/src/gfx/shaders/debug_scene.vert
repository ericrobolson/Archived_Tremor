#version 450

#define MAX_NUM_JOINTS 128

layout (location = 0) in vec3 in_pos;
layout (location = 1) in vec4 in_color;

layout (set = 0, binding = 0) uniform SceneUbo
{
	mat4 view_projection;
	mat4 projection;
	mat4 view;
	vec3 cam_pos;
} ubo;

layout (location = 0) out vec4 out_color;

out gl_PerVertex
{
	vec4 gl_Position;
};

void main() 
{
	out_color = in_color;
	gl_Position =  ubo.view_projection * vec4(in_pos, 1.0);
}

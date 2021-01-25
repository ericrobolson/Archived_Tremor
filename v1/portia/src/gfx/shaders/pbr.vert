#version 450

#define MAX_NUM_JOINTS 128

layout (location = 0) in vec3 in_pos;
layout (location = 1) in vec3 in_normal;
layout (location = 2) in vec2 in_uv0;
layout (location = 3) in vec2 in_uv1;
layout (location = 4) in vec4 in_joint0;
layout (location = 5) in vec4 in_weight0;

// TODO: port UBO nodes + mat4 model to instance data?

layout (set = 0, binding = 0) uniform SceneUbo
{
	mat4 view_projection;
	mat4 projection;
	mat4 view;
	vec3 cam_pos;
} ubo;

layout(set=1, binding = 0) uniform ModelUbo{
	mat4 model;
} model_uniforms;

layout (set = 2, binding = 0) uniform NodeUbo {
	mat4 matrix;
	mat4 joint_matrix[MAX_NUM_JOINTS];
	float joint_count; // Does this really need to be a float? Update UniformBlock if not.
} node;

layout (location = 0) out vec3 out_world_pos;
layout (location = 1) out vec3 out_normal;
layout (location = 2) out vec2 out_uv0;
layout (location = 3) out vec2 out_uv1;

out gl_PerVertex
{
	vec4 gl_Position;
};

void main() 
{
	vec4 loc_pos;
	if (node.joint_count > 0.0) {
		// Mesh is skinned
		mat4 skin_mat = 
			in_weight0.x * node.joint_matrix[int(in_joint0.x)] +
			in_weight0.y * node.joint_matrix[int(in_joint0.y)] +
			in_weight0.z * node.joint_matrix[int(in_joint0.z)] +
			in_weight0.w * node.joint_matrix[int(in_joint0.w)];

		loc_pos = model_uniforms.model * node.matrix * skin_mat * vec4(in_pos, 1.0);
		out_normal = normalize(transpose(inverse(mat3(model_uniforms.model * node.matrix * skin_mat))) * in_normal);
	} else {
		loc_pos = model_uniforms.model * node.matrix * vec4(in_pos, 1.0);
		out_normal = normalize(transpose(inverse(mat3(model_uniforms.model * node.matrix))) * in_normal);
	}
	loc_pos.y = -loc_pos.y;
	out_world_pos = loc_pos.xyz / loc_pos.w;
	out_uv0 = in_uv0;
	out_uv1 = in_uv1;
	gl_Position =  ubo.view_projection * vec4(out_world_pos, 1.0);
}

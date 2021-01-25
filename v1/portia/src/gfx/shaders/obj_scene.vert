#version 450

// Inputs
layout(location=0) in vec3 a_position;
layout(location=1) in vec2 a_tex_coords;
layout(location=2) in vec3 a_normal;
layout(location=3) in vec3 a_tangent;
layout(location=4) in vec3 a_bitangent;
layout(location=5) in mat4 model_matrix;

// Uniforms
layout(set=0, binding=0)
uniform Uniforms{
   // vec2 u_viewport_size;
   // vec2 u_screen_size;
    mat4 u_view_proj;
    vec3 u_view_pos;
    mat4 u_view;
};

// Outputs
layout(location=0) out vec2 v_tex_coords;
layout(location=1) out vec3 v_normal;
layout(location=2) out vec3 frag_pos;
layout(location=3) out mat3 v_tbn;


void main() {
    // Textures
    v_tex_coords = a_tex_coords;

    // Normals (https://learnopengl.com/Advanced-Lighting/Normal-Mapping). Added in Gram-Schmidt process.
    vec3 t = normalize(vec3(model_matrix * vec4(a_tangent,   0.0)));
    vec3 n = normalize(vec3(model_matrix * vec4(a_normal,    0.0)));
    t = normalize(t - dot(t, n) * n);
    vec3 b = cross(n, t);
    v_tbn = mat3(t,b,n);

    mat3 norm_matrix = mat3(transpose(inverse(model_matrix))); // TODO: this is expensive and should be offloaded to the instance data.
    v_normal = norm_matrix * a_normal;

    // Positions
    gl_Position = u_view_proj * model_matrix * vec4(a_position, 1.0);
    frag_pos = vec3(model_matrix * vec4(a_position, 1.0));
}

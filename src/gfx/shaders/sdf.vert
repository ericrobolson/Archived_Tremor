#version 450

out gl_PerVertex {
    vec4 gl_Position;
};

// Rather lame, but this will draw two triangles on the screen so that the frag shader can draw SDF's. 
const vec2 positions[6] = vec2[6](
    // tri 1
    vec2(-1.0, 1.0),
    vec2(-1.0, -1.0),
    vec2(1.0, -1.0),
    // tri 2
    vec2(1.0, -1.0),
    vec2(1.0, 1.0),
    vec2(-1.0, 1.0)
);


void main() {
    gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
}

#version 450

// Inputs
layout (location = 0) in vec4 in_color;

// Outputs
layout(location=0) out vec4 f_color;

void main() {
    f_color = in_color;
}
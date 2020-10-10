#version 450

#define MAX_STEPS 100
#define MAX_DIST 100.0
#define SURFACE_DIST 0.01

layout(location=0) out vec4 f_color;


float sphereSdf(vec3 p) {
    return length(p) - 1.0;
}

float rayMarch(vec3 ro, vec3 rd) {
    float depth = 0.0;
    for (int i = 0; i < MAX_STEPS; i++){
        
    }

    return depth;
}

void main(){
    f_color = vec4(gl_FragCoord.x, gl_FragCoord.y, gl_FragCoord.z, 1.0);

}
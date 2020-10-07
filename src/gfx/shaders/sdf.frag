#version 450

#define MAX_STEPS 100
#define MAX_DIST 100.0
#define SURFACE_DIST 0.01


float sphereSdf(vec3 p) {
    return length(p) - 1.0;
}

float rayMarch(vec3 ro, vec3 rd) {
    float d0 = 0.0;
    for (int i = 0; i < MAX_STEPS; i++){

    }

    return d0;
}

void main(){
}
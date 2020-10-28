#version 450

#define MAX_STEPS 100
#define MAX_DIST 1000.0
#define SURFACE_DIST 0.01

#define VOXEL_BUF_LEN 420

layout(location=0) out vec4 f_color;


layout(set=0, binding=0)
uniform Uniforms{
    vec3 u_view_position;
    mat4 u_view_proj;
    vec2 u_viewport_size;
};


layout(set=1, binding=0)
uniform VoxelUniforms{
    float voxel_resolution;
    mat3 voxel_world_size;
    vec3 world_size;
};

layout(set=2, binding = 0) uniform texture3D volume_tex;
layout(set=2, binding=1) uniform sampler volume_sampler;


float VoxelVolumeSdf(vec3 point) {
    // TODO: Convert point into a [0..1,0..1,0..1] point in voxel space. 
    // TODO: get the texture item at that point. If it's a hit, return min dist. If it's not, return voxel resolution.

    return MAX_DIST; //TODO: fix up
}

float sphereSdf(vec3 p, vec3 spherePos, float radius) {
    return length(p - spherePos) - radius;
}

float boxSdf(vec3 point, vec3 boxPos, vec3 box){
    vec3 p = point;
    vec3 q = abs(p) - box ;
    return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
}

float boxSdf(vec3 point, vec3 box){
    vec3 q = abs(point) - box;
    return length(max(q,0.0)) + min(max(q.x,max(q.y,q.z)),0.0);
}

bool in_bounds(float p, float min, float max) {
    return p <= max && p >= min;
}

bool in_bounds(vec3 point, vec3 min, vec3 max){
    return 
        in_bounds(point.x, min.x, max.x)
        && in_bounds(point.y, min.y, max.y)
        && in_bounds(point.z, min.z, max.z)
        ;
}

float volumeSdf(vec3 point) {
    float bound = 20;
    vec3 max = vec3(bound, bound, bound);
    vec3 min = -max;

    // Idea: if point out of bounds, return boxSdf + the projection INTO the volume texture
    
    if (in_bounds(point, min, max)){
        return float(texture(sampler3D(volume_tex, volume_sampler), normalize(point)));;
    } else {
        vec3 dist2 = vec3(0,0,0);
        if (point.x <= min.x) {
            dist2.x = min.x;
        } else {
            dist2.x = max.x;
        }

        if (point.y <= min.y) {
            dist2.y = min.y;
        } else {
            dist2.y = max.y;
        }

        if (point.z <= min.z) {
            dist2.z = min.z;
        } else {
            dist2.z = max.z;
        }

        return length(point - dist2);
    }
}

float mandelbulb(vec3 point){ 
    vec3 w = point;
    float m = dot(w, w);
    float dz = 1.0;
    for (int i = 0; i < 32; i++){
        dz = 8 * pow(sqrt(m), 7.0) * dz + 1.0;
        float r = length(w);
        float b = 8 * acos(w.y / r);
        float a = 8 * atan(w.x, w.z);
        w = point + pow(r, 8) * vec3(sin(b) * sin(a), cos(b), sin(b) * cos(a));

        m = dot(w, w);
		if(m > 256.0)
            break;

    }

    return 0.25*log(m)*sqrt(m)/dz;
}

float GetDist(vec3 point){
    float dPlane = point.y - 1; // Ground plane at 0

    float sceneDist = MAX_DIST;
    sceneDist = min(sceneDist, volumeSdf(point));
    
    return sceneDist;
}

float RayMarch(vec3 rayOrigin, vec3 rayDirection) {
    float distanceOrigin = 0.0;

    for (int i = 0; i < MAX_STEPS; i++){
        vec3 point = rayOrigin + distanceOrigin * rayDirection;
        float distanceScene = GetDist(point);
        distanceOrigin += distanceScene;
        if (distanceScene < SURFACE_DIST || distanceOrigin > MAX_DIST) {
            break;
        }
    }

    return distanceOrigin;
}

vec3 GetNormal(vec3 point){
    float distance = GetDist(point);
    
    // Get a few points around the point to calculate the normal
    vec2 e = vec2(0.1, 0);
    vec3 normal = distance - vec3(
        GetDist(point - e.xyy),
        GetDist(point - e.yxy),
        GetDist(point - e.yyx)
    );

    return normalize(normal);
}

float GetLight(vec3 point) {
    vec3 lightPosition = vec3(1, 5, 6);
    vec3 light = normalize(lightPosition - point);
    vec3 normal = GetNormal(point);

    float diffuseLight = clamp(dot(normal, light), 0.0, 1.0);

    // Shadow TODO: fix issue with misplaced shadows on SDFs    
    
    float dist = RayMarch(point + normal * SURFACE_DIST * 2.0, vec3(1.0));
    if (dist < length(lightPosition - point)) {
        diffuseLight *= 0.1;
    }
    
    return diffuseLight;
}

void main(){
    vec2 fragCoord = gl_FragCoord.xy;

    vec2 uv = (fragCoord - 0.5 * u_viewport_size) / -u_viewport_size.y; // for some reason, viewport is flipped so flip it to the 'normal' view. 
    
    vec3 col = vec3(0);
    vec3 rayOrigin = u_view_position;
    vec3 rayDistance = normalize(vec3(uv.x, uv.y, 1));

    float dist = RayMarch(rayOrigin, rayDistance);

    if (dist >= MAX_DIST){
        discard;
        return;
    }

    vec3 point = rayOrigin + rayDistance * dist;
    float diffuseLight = GetLight(point);

    col = vec3(diffuseLight);

    f_color = vec4(col, 1.0);
}
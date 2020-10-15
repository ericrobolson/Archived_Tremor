#version 450

#define MAX_STEPS 100
#define MAX_DIST 1000.0
#define SURFACE_DIST 0.01

#define VOXEL_BUF_LEN 128

layout(location=0) out vec4 f_color;


layout(set=0, binding=0)
uniform Uniforms{
    vec3 u_view_position;
    mat4 u_view_proj;
    vec2 u_viewport_size;
};

layout(set=1, binding = 0) buffer voxelStream{ 
    uint elements[VOXEL_BUF_LEN];
};

float sphereSdf(vec3 p, vec3 spherePos, float radius) {
    return length(p - spherePos) - radius;
}

float boxSdf(vec3 point, vec3 boxPos, vec3 box){
    vec3 p = point;
    vec3 q = abs(p) - box ;
    return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
}

float voxelOctreeSdf(vec3 point){
    float boxMin = 100000.0;

    vec3 octree_pos = vec3(0, 1, 6);

    // The 'span' of an octree, from one end to another
    float octree_span = 4.0; 

    int index = 0;
    while (index < VOXEL_BUF_LEN){
        // Get nearest octree to point 
        // If empty, get next octree  

        /*
        Psuedo code:
            get nearest voxel in octree to point. Return distance from it to the point. 
        */

        index += 1;
    }

    return 100000.0;
    /*
    for (int i = 0; i < VOXEL_BUF_LEN; i++){ //TODO: iterate over buff len?
        // TODO: read things from buffer
        uint e = elements[i];       
    }
    */
}

float GetDist(vec3 point){
    vec3 spherePosition = vec3(0, 1, 6);
    float sphereRadius = 1;
    float sphereDistance = sphereSdf(point, spherePosition, sphereRadius);
    float dPlane = point.y; // Ground plane at 0

    float sceneDist = min(sphereDistance, dPlane);

    float voxelDist = voxelOctreeSdf(point);


    
    return min(sceneDist, voxelDist);
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
    /*
    float dist = RayMarch(point + normal * SURFACE_DIST * 2.0, vec3(1.0));
    if (dist < length(lightPosition - point)) {
        diffuseLight *= 0.1;
    }
    */
    

    return diffuseLight;
}

void main(){
    vec2 fragCoord = gl_FragCoord.xy;

    vec2 uv = (fragCoord - 0.5 * u_viewport_size) / -u_viewport_size.y; // for some reason, viewport is flipped so flip it to the 'normal' view. 
    
    vec3 col = vec3(0);
    vec3 rayOrigin = u_view_position;
    vec3 rayDistance = normalize(vec3(uv.x, uv.y, 1));

    float dist = RayMarch(rayOrigin, rayDistance);
    vec3 point = rayOrigin + rayDistance * dist;
    float diffuseLight = GetLight(point);

    col = vec3(diffuseLight);

    f_color = vec4(col, 1.0);
}
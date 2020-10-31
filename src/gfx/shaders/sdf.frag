#version 450
#extension GL_EXT_samplerless_texture_functions: require

#define MAX_STEPS 200 // TODO: convert to runtime variable
#define MAX_DIST 2000.0 // TODO: convert to runtime variable
#define SURFACE_DIST 0.01 // TODO: convert to runtime variable

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
    vec3 voxel_world_size;
};

layout(set=2, binding=0) uniform utexture3D volume_tex;
layout(set=2, binding=1) uniform usampler3D volume_sampler;




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

vec4 map_color(uint r, uint g, uint b){
    return vec4(r / 255.0, g / 255.0, b / 255.0, 1.0);
}

float voxel_volume_sdf(vec3 point, out vec4 color) {
    color = vec4(0);

    ivec3 vp = ivec3(point / voxel_resolution); // scale point to texture position

    uint raw_value = texelFetch(volume_tex, vp, 0).r;
    bool not_empty = raw_value > 0u;
    uint value = raw_value >> 1;
    if (not_empty) {
        uint mat_value = value;
        //TODO: get color value from texture. This be bad. 

        if (raw_value == 1u) {
            color = map_color(252, 215, 172);
        } else if (raw_value == 2u) {
            color = map_color(255, 244, 232);
        } else if (raw_value == 3u) {
            // Why is only this one showing?
            color = map_color(99, 216, 255);
        } else if (raw_value == 4u) {
            color = map_color(83, 94, 97);
        }


        return SURFACE_DIST;
    } 
    
    // Now calculate the distance to the next voxel
    float distance = value * voxel_resolution;


    return voxel_resolution * 0.9; //TODO: instead return dist
}

float GetDist(vec3 point, out vec4 color){
    color = vec4(0);
    float dPlane = point.y - 1; // Ground plane at 0

    float sceneDist = dPlane;
  
    sceneDist = min(sceneDist, sphereSdf(point, vec3(0,3,10), 1));

    float voxel_dist = voxel_volume_sdf(point, color);
    sceneDist = min(sceneDist, voxel_dist);

    
    return sceneDist;
}
float mincomp( vec3 v ) { return min( min( v.x, v.y ), v.z ); }


float RayMarch(vec3 rayOrigin, vec3 rayDirection, out vec4 color) {
    float distanceOrigin = 0.0;

    for (int i = 0; i < MAX_STEPS; i++){
        vec3 point = rayOrigin + distanceOrigin * rayDirection;
        float distanceScene = GetDist(point, color);
        distanceOrigin += distanceScene;
        if (distanceScene < SURFACE_DIST || distanceOrigin > MAX_DIST) {
            break;
        }
    }

    return distanceOrigin;
}

vec3 GetNormal(vec3 point){
    vec4 color;
    float distance = GetDist(point,color);
    
    // Get a few points around the point to calculate the normal
    vec2 e = vec2(0.1, 0);
    vec3 normal = distance - vec3(
        GetDist(point - e.xyy, color),
        GetDist(point - e.yxy, color),
        GetDist(point - e.yyx, color)
    );

    return normalize(normal);
}

float GetLight(vec3 point) {
    vec3 lightPosition = vec3(1, 25, 6);
    vec3 light = normalize(lightPosition - point);
    vec3 normal = GetNormal(point);

    float diffuseLight = clamp(dot(normal, light), 0.0, 1.0);

    // Shadow TODO: fix issue with misplaced shadows on SDFs    
    
    vec4 color;
    float dist = RayMarch(point + normal * SURFACE_DIST * 2.0, vec3(1.0), color);
    if (dist < length(lightPosition - point)) {
        diffuseLight *= 0.1;
    }
    
    return diffuseLight;
}

void main(){
    vec2 fragCoord = gl_FragCoord.xy;

    vec2 uv = (fragCoord - 0.5 * u_viewport_size) / -u_viewport_size.y; // for some reason, viewport is flipped so flip it to the 'normal' view. 
    
    vec3 rayOrigin = u_view_position;
    vec3 rayDirection = normalize(vec3(uv.x, uv.y, 1));
    vec4 color;
    float dist = RayMarch(rayOrigin, rayDirection, color);

 

    // Simple flatshading
    //f_color = color;

   if (dist >= MAX_DIST){
        discard;
        return;
    }

    //TODO: add in lighting 

    bool do_lighting = false;
    if (do_lighting){
        vec3 point = rayOrigin + rayDirection * dist;
        float diffuseLight = GetLight(point);

        color *= vec4(vec3(diffuseLight), 1);
    }
   

    f_color = color;
}
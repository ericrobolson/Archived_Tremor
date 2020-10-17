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

    vec3 octree_pos = vec3(0, 2, 8);

    // The 'span' of an octree, from one end to another
    float octree_span = 4.0; 

    int index = 0;
    float dist_to_nearest_voxel = MAX_DIST;
    while (index < 3){ // just testing n = 2 levels; Need to actually figure this out
        uint node = elements[index]; // Read from buffer
        uint child_pointer = node >> (1 + 8 + 8); // Get the child pointer by rshifting the farbit, valid and leaf masks out

        bool is_header = index == 0; //TODO: need to figure out how to handle the headers. Should we be iterating over the stream like this?
        if (is_header) {
            index++;
            continue;
        }

        // Simple algorithm for octree level 1 deep
        // For each non empty 'child', get the distances to the point. Return whichever one is smaller. 
        // Step 1: if leaf, do a SDF for a box on that leaf's position and size. 
        // Step 2: if step 1 returns less than the previous distance, register that. Otherwise break. 
        
        // Iterate over each child, storing the distance 
        float leaf_dist = MAX_DIST;
        for (int i = 0; i < 8; i++) { // TODO: unroll this into a separate function 
            // Get bit at i. Skip if not active
            bool valid_mask = (( 1 << i) & node) != 0;
            bool leaf_mask = (( 1 << (i + 8)) & node) != 0;

            if (!valid_mask) {
                continue;
            }
            

            // TODO: how to deal with children?

            // Calculate SDF (box) at this child's size + child's position
            float sdf_bounds = octree_span / 2.0; // Each level we go, divide the bounds further. Done by 2 as the 'span' of a octree level is 2 items.
            vec3 sdf_pos = octree_pos; 

            // Calculate the positions for the current octree child            
            // Alternate x if even or odd
            /*
            if (i % 2 == 0) {
                sdf_pos.x += sdf_bounds / 2.0;
            } else {
                sdf_pos.x -= sdf_bounds / 2.0;
            }
            */
            
            sdf_pos.x += (sdf_bounds / 2.0) * (((i % 2) * -2) + 1);

            // TODO: remove the branching here; replace with mathmatical functions

            // Alternate y if >= 4
            if (i < 4) {
                sdf_pos.y += sdf_bounds / 2.0;
            } else {
                sdf_pos.y -= sdf_bounds / 2.0;
            }
            // Alternate z if 0..1 or 2..3
            if (i % 4 <= 1) {
                sdf_pos.z += sdf_bounds / 2;
            } else {
                sdf_pos.z -= sdf_bounds / 2;
            }


             // TODO: convert to box if you want boxy voxels. 
            float sphere_radius = sdf_bounds / 2.0; // Divide by 2 as spheres have a radius, not a diameter
            float sdf =  sphereSdf(point, sdf_pos, sphere_radius);

            if (sdf < leaf_dist) {
                leaf_dist = sdf;
            }
        }

        if (leaf_dist < dist_to_nearest_voxel) {
            dist_to_nearest_voxel = leaf_dist;
        }

        index++;
    }

    return dist_to_nearest_voxel;
}

float GetDist(vec3 point){
    vec3 spherePosition = vec3(0, 1, 6);
    float sphereRadius = 1;
    float sphereDistance = MAX_DIST;// sphereSdf(point, spherePosition, sphereRadius);
    float dPlane = point.y; // Ground plane at 0

    float sceneDist = dPlane;

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
    vec3 point = rayOrigin + rayDistance * dist;
    float diffuseLight = GetLight(point);

    col = vec3(diffuseLight);

    f_color = vec4(col, 1.0);
}
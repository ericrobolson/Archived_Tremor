#version 450

// Inputs
layout(location=0) in vec2 v_tex_coords;
layout(location=1) in vec3 v_normal;
layout(location=2) in vec3 v_frag_position;
layout(location=3) in mat3 v_tbn;

// Uniforms
layout(set=0, binding=0)
uniform Uniforms{
    mat4 u_view_proj;
    vec3 u_view_pos;
    mat4 u_view;
};

layout(set=1, binding=0) uniform texture2D t_diffuse;
layout(set=1, binding=1) uniform sampler s_diffuse;

layout(set=1, binding=2) uniform texture2D t_normal;
layout(set=1, binding=3) uniform sampler s_normal;


// Outputs
layout(location=0) out vec4 f_color;


struct Material{
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    float shininess;
};

struct DirectionalLight {
    vec3 direction;
    vec3 ambient;
    float ambient_strength;
    vec3 diffuse;
    vec3 specular;
};

struct PointLight {
    vec3 position;

    vec3 ambient;
    float ambient_strength;

    vec3 diffuse;
    vec3 specular;
    
    float constant;
    float linear;
    float quadratic;
};


vec3 specular(vec3 view_dir, vec3 light_dir, vec3 normal, float mat_shininess, vec3 mat_specular, vec3 l_specular){
    vec3 half_dir = normalize(view_dir + light_dir);
    float spec = pow(max(dot(normal, half_dir), 0.0), mat_shininess);
    vec3 specular = vec3(mat_specular * (spec * l_specular));
    return specular;
}

vec3 diffuse(vec3 light_dir, vec3 normal, vec3 mat_diffuse, vec3 l_diffuse){
    float diff = max(dot(normal, light_dir), 0.0);
    vec3 diffuse = vec3((diff * mat_diffuse) * l_diffuse);
    return diffuse;
}

vec3 ambient(vec3 l_ambient, float l_ambient_str, vec3 mat_ambient){
    vec3 ambient = vec3(l_ambient * l_ambient_str * mat_ambient);
    return ambient;
}

vec3 directional_light(DirectionalLight light, Material material, vec3 normal, vec3 view_dir){
    vec3 light_dir = normalize(light.direction);

    // shading
    vec3 diffuse = diffuse(light_dir, normal, material.diffuse, light.diffuse);
    vec3 specular = specular(view_dir, light_dir, normal, material.shininess, material.specular, light.specular);
    vec3 ambient = ambient(light.ambient, light.ambient_strength, material.ambient);

    return ambient + diffuse + specular;
}

vec3 point_light(PointLight light, Material material, vec3 normal, vec3 frag_pos, vec3 view_dir)
{
    vec3 light_dir = normalize(light.position - frag_pos);

    // shading
    vec3 diffuse = diffuse(light_dir, normal, material.diffuse, light.diffuse);
    vec3 specular = specular(view_dir, light_dir, normal, material.shininess, material.specular, light.specular);
    vec3 ambient = ambient(light.ambient, light.ambient_strength, material.ambient);

    // attenuation
    float distance    = length(light.position - frag_pos);
    float attenuation = 1.0 / (light.constant + light.linear * distance + 
  			     light.quadratic * (distance * distance));    

    // combine results
    ambient  *= attenuation;
    diffuse  *= attenuation;
    specular *= attenuation;

    return (ambient + diffuse + specular);
} 

void main(){
    Material material = Material(vec3(1.0, 1.0, 1.0), vec3(1.0, 1.0, 1.0), vec3(0.5, 0.5, 0.5), 32.0); 
    DirectionalLight directional_l = DirectionalLight(vec3(0.0, 4.0, 0.0), vec3(0.1, 0.1, 0.1), 0.01, vec3(0.1, 0.1, 0.1), vec3(1.0, 1.0, 1.0));

    vec4 tex_color = texture(sampler2D(t_diffuse, s_diffuse), v_tex_coords);
    vec3 tex_normal = texture(sampler2D(t_normal, s_normal), v_tex_coords).rgb;

    // Lighting prep
    vec3 norm = tex_normal * 2.0 - 1.0;
    norm = normalize(v_tbn * norm);
    vec3 view_dir = normalize(u_view_pos - v_frag_position);

    // Lighting calculation 
    vec3 light = vec3(0.0, 0.0, 0.0);

    // Directional
    vec3 directional = directional_light(directional_l, material, norm, view_dir);
    light += directional;

    // Points
    for (int i = 0; i < 4; i++){
        PointLight plight = PointLight(vec3(float(i) * 100.0, 200.0, 0.0), vec3(0.0, 0.0, 0.0), 0.0, vec3(0.5, 0.5, 0.5), vec3(1.0, 1.0, 1.0), 1.0, 0.009, 0.0032);
        vec3 point_l = point_light(plight, material, norm,v_frag_position, view_dir);

        light += point_l;
    }   

    // Assign colors
    vec4 final_light = vec4(light, 1.0);
    f_color = final_light * tex_color;
}
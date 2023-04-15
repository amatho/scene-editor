#version 410 core

in vec2 tex_coords;

out vec4 out_frag_color;

struct DirLight {
    vec3 direction;

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

struct PointLight {
    vec3 position;

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;

    float constant;
    float linear;
    float quadratic;
};

uniform sampler2D position_tx;
uniform sampler2D normal_tx;
uniform sampler2D albedo_spec_tx;

uniform vec3 view_pos;
uniform mat4 light_space_matrix;

uniform DirLight dir_light;
#define MAX_POINT_LIGHTS 128
uniform int point_lights_size;
uniform PointLight point_lights[MAX_POINT_LIGHTS];

uniform sampler2DShadow shadow_map_tx;

vec3 calculate_general_light(vec3 light_ambient, vec3 light_diffuse, vec3 light_specular, vec3 light_dir, vec3 normal, vec3 albedo, float specular_strength, vec3 view_dir, float shadow) {
    float diff = max(dot(normal, light_dir), 0.0);
    vec3 halfway_dir = normalize(light_dir + view_dir);
    float spec = pow(max(dot(normal, halfway_dir), 0.0), 16.0);

    vec3 ambient = light_ambient * albedo;
    vec3 diffuse = light_diffuse * diff * albedo;
    vec3 specular = light_specular * spec * specular_strength;

    return ambient + shadow * (diffuse + specular);
}

vec3 calculate_dir_light(vec3 normal, vec3 albedo, float specular_strength, vec3 view_dir, float shadow) {
    vec3 light_dir = normalize(-dir_light.direction);
    return calculate_general_light(dir_light.ambient, dir_light.diffuse, dir_light.specular, light_dir, normal, albedo, specular_strength, view_dir, shadow);
}

vec3 calculate_point_light(PointLight light, vec3 frag_pos, vec3 normal, vec3 albedo, float specular_strength, vec3 view_dir) {
    vec3 light_dir = normalize(light.position - frag_pos);
    float distance = length(light.position - frag_pos);
    float attenuation = 1.0 / (light.constant + light.linear * distance + light.quadratic * (distance * distance));

    vec3 color = calculate_general_light(light.ambient, light.diffuse, light.specular, light_dir, normal, albedo, specular_strength, view_dir, 1.0);
    color *= attenuation;

    return color;
}

float calculate_shadow(vec4 frag_pos_light_space, vec3 normal) {
    vec3 proj_coords = frag_pos_light_space.xyz / frag_pos_light_space.w;
    proj_coords = proj_coords * 0.5 + 0.5;

    if (proj_coords.z > 1.0) {
        return 1.0;
    }

    float current_depth = proj_coords.z;
    vec3 light_dir = normalize(-dir_light.direction);
    float bias = clamp(0.005 * tan(acos(max(dot(normal, light_dir), 0.0))), 0.0, 0.01);

    float shadow = 0.0;
    vec2 texel_size = 1.0 / textureSize(shadow_map_tx, 0);
    for (float y = -1.5; y <= 1.5; y += 1.0) {
        for (float x = -1.5; x <= 1.5; x += 1.0) {
            shadow += texture(shadow_map_tx, vec3(proj_coords.xy + vec2(x, y) * texel_size, current_depth - bias));
        }
    }
    shadow /= 16.0;

    return shadow;
}

void main() {
    vec4 pos = texture(position_tx, tex_coords);
    vec3 frag_pos = pos.rgb;

    if (pos.a == 1.0) {
        out_frag_color = vec4(1.0, 0.5, 0.0, 1.0);
        return;
    }

    vec3 normal = texture(normal_tx, tex_coords).rgb;

    if (normal == vec3(0.0, 0.0, 0.0)) {
        out_frag_color = vec4(0.4, 0.4, 1.0, 1.0);
        return;
    }

    vec3 albedo = texture(albedo_spec_tx, tex_coords).rgb;
    float specular = texture(albedo_spec_tx, tex_coords).a;

    vec3 view_dir = normalize(view_pos - frag_pos);
    vec3 result = vec3(0.0);

    float shadow = calculate_shadow(light_space_matrix * vec4(frag_pos, 1.0), normal);
    result += calculate_dir_light(normal, albedo, specular, view_dir, shadow);

    int size = min(point_lights_size, MAX_POINT_LIGHTS);
    for (int i = 0; i < size; i++) {
        result += calculate_point_light(point_lights[i], frag_pos, normal, albedo, specular, view_dir);
    }

    out_frag_color = vec4(result, 1.0);
}

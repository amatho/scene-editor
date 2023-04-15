#version 410 core

in vec3 frag_pos;
in vec3 normal;
in vec2 tex_coords;

layout(location = 0) out vec4 out_position;
layout(location = 1) out vec3 out_normal;
layout(location = 2) out vec4 out_albedo_spec;

uniform sampler2D diffuse_tx;
uniform sampler2D specular_tx;
uniform float selected;

void main() {
    out_position = vec4(frag_pos, selected);
    out_normal = normalize(normal);
    out_albedo_spec.rgb = texture(diffuse_tx, tex_coords).rgb;
    out_albedo_spec.a = texture(specular_tx, tex_coords).r;
}

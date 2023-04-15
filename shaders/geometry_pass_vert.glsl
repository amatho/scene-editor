#version 410 core

layout(location = 0) in vec3 in_pos;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in vec2 in_tex_coords;

out vec3 frag_pos;
out vec3 normal;
out vec2 tex_coords;

uniform mat4 mvp;
uniform mat4 model;
uniform mat3 normal_mat;

void main() {
    frag_pos = vec3(model * vec4(in_pos, 1.0));
    normal = normal_mat * in_normal;
    tex_coords = in_tex_coords;

    gl_Position = mvp * vec4(in_pos, 1.0);
}

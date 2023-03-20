#version 410 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal_in;
layout(location = 2) in vec2 texCoord_in;

uniform mat4 mvp;

layout(location = 0) out vec3 normal_out;
layout(location = 1) out vec2 texCoord_out;

void main() {
    normal_out = normalize(normal_in);
    texCoord_out = texCoord_in;

    gl_Position = mvp * vec4(position, 1.0);
}

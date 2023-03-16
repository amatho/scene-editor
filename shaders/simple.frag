#version 410 core

layout(location = 0) in vec3 normal;
layout(location = 1) in vec2 texCoord;

out vec4 color;

void main() {
    color = vec4(1.0, 0.0, 0.0, 1.0);
}

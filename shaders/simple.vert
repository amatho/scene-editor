#version 410 core

layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aNormal;
layout(location = 2) in vec2 aTexCoord;

uniform mat4 mvp;

out vec3 Normal;
out vec2 TexCoord;

void main() {
    Normal = normalize(aNormal);
    TexCoord = aTexCoord;

    gl_Position = mvp * vec4(aPos, 1.0);
}
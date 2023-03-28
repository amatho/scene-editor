#version 410 core

in vec3 Normal;
in vec2 TexCoord;

out vec4 color;

void main() {
    color = vec4(Normal, 1.0);
}

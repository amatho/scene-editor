#version 410 core

in vec3 Normal;
in vec2 TexCoord;

out vec4 color;

uniform sampler2D tex;

void main() {
    color = texture(tex, TexCoord);
}

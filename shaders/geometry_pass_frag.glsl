#version 410 core

in vec3 FragPos;
in vec3 Normal;
in vec2 TexCoords;

layout(location = 0) out vec4 gPosition;
layout(location = 1) out vec3 gNormal;
layout(location = 2) out vec4 gAlbedoSpec;

uniform sampler2D diffuse;
uniform sampler2D specular;
uniform float selected;

void main() {
    gPosition = vec4(FragPos, selected);
    gNormal = normalize(Normal);
    gAlbedoSpec.rgb = texture(diffuse, TexCoords).rgb;
    gAlbedoSpec.a = texture(specular, TexCoords).r;
}

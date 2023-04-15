#version 410 core

in vec2 TexCoords;

out vec4 FragColor;

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

uniform sampler2D gPosition;
uniform sampler2D gNormal;
uniform sampler2D gAlbedoSpec;

uniform vec3 viewPos;
uniform mat4 lightSpaceMatrix;

uniform DirLight dirLight;
#define MAX_POINT_LIGHTS 128
uniform int pointLightsSize;
uniform PointLight pointLights[MAX_POINT_LIGHTS];

uniform sampler2DShadow shadowMap;

vec3 CalculateDirLight(DirLight light, vec3 normal, vec3 albedo, float specularStrength, vec3 viewDir, float shadow);
vec3 CalculatePointLight(PointLight light, vec3 fragPos, vec3 normal, vec3 albedo, float specularStrength, vec3 viewDir);
vec3 CalculateGeneralLight(vec3 ambient, vec3 diffuse, vec3 specular, vec3 lightDir, vec3 normal, vec3 albedo, float specularStrength, vec3 viewDir, float shadow);
float CalculateShadow(vec4 fragPosLightSpace, vec3 normal);

void main() {
    vec4 pos = texture(gPosition, TexCoords);
    vec3 FragPos = pos.rgb;

    if (pos.a == 1.0) {
        FragColor = vec4(1.0, 0.5, 0.0, 1.0);
        return;
    }

    vec3 Normal = texture(gNormal, TexCoords).rgb;

    if (Normal == vec3(0.0, 0.0, 0.0)) {
        FragColor = vec4(0.4, 0.4, 1.0, 1.0);
        return;
    }

    vec3 Albedo = texture(gAlbedoSpec, TexCoords).rgb;
    float Specular = texture(gAlbedoSpec, TexCoords).a;

    vec3 viewDir = normalize(viewPos - FragPos);
    vec3 result = vec3(0.0);

    float shadow = CalculateShadow(lightSpaceMatrix * vec4(FragPos, 1.0), Normal);
    result += CalculateDirLight(dirLight, Normal, Albedo, Specular, viewDir, shadow);

    for (int i = 0; i < pointLightsSize; i++) {
        result += CalculatePointLight(pointLights[i], FragPos, Normal, Albedo, Specular, viewDir);
    }

    FragColor = vec4(result, 1.0);
}

vec3 CalculateDirLight(DirLight light, vec3 normal, vec3 albedo, float specularStrength, vec3 viewDir, float shadow) {
    vec3 lightDir = normalize(-light.direction);
    return CalculateGeneralLight(light.ambient, light.diffuse, light.specular, lightDir, normal, albedo, specularStrength, viewDir, shadow);
}

vec3 CalculatePointLight(PointLight light, vec3 fragPos, vec3 normal, vec3 albedo, float specularStrength, vec3 viewDir) {
    vec3 lightDir = normalize(light.position - fragPos);
    float distance = length(light.position - fragPos);
    float attenuation = 1.0 / (light.constant + light.linear * distance + light.quadratic * (distance * distance));

    vec3 color = CalculateGeneralLight(light.ambient, light.diffuse, light.specular, lightDir, normal, albedo, specularStrength, viewDir, 1.0);
    color *= attenuation;

    return color;
}

vec3 CalculateGeneralLight(vec3 lightAmbient, vec3 lightDiffuse, vec3 lightSpecular, vec3 lightDir, vec3 normal, vec3 albedo, float specularStrength, vec3 viewDir, float shadow) {
    float diff = max(dot(normal, lightDir), 0.0);
    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), 16.0);

    vec3 ambient = lightAmbient * albedo;
    vec3 diffuse = lightDiffuse * diff * albedo;
    vec3 specular = lightSpecular * spec * specularStrength;

    return ambient + shadow * (diffuse + specular);
}

float CalculateShadow(vec4 fragPosLightSpace, vec3 normal) {
    vec3 projCoords = fragPosLightSpace.xyz / fragPosLightSpace.w;
    projCoords = projCoords * 0.5 + 0.5;

    if (projCoords.z > 1.0) {
        return 1.0;
    }

    float currentDepth = projCoords.z;
    vec3 lightDir = normalize(-dirLight.direction);
    float bias = clamp(0.005 * tan(acos(max(dot(normal, lightDir), 0.0))), 0.0, 0.01);

    float shadow = 0.0;
    vec2 texelSize = 1.0 / textureSize(shadowMap, 0);
    for (float y = -1.5; y <= 1.5; y += 1.0) {
        for (float x = -1.5; x <= 1.5; x += 1.0) {
            shadow += texture(shadowMap, vec3(projCoords.xy + vec2(x, y) * texelSize, currentDepth - bias));
        }
    }
    shadow /= 16.0;

    return shadow;
}

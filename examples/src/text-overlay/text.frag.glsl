
#version 450 core
#extension GL_ARB_separate_shader_objects : enable

#define CHARACTER_COUNT 128

layout (location = 0) in vec2 inUV;
layout (location = 1) in vec4 inColor;
layout (location = 2) flat in uint inCharacterId; // flat key word disable interpolation.

layout (location = 0) out vec4 outColor;

layout (binding = 0) uniform sampler textSampler;
layout (binding = 1) uniform texture2D glyphTextures[CHARACTER_COUNT];

void main() {

    vec4 color = inColor * texture(sampler2D(glyphTextures[inCharacterId], textSampler), inUv);

    if (color.a <= 0.3) {
        discard;
    }

    outColor = color;
}

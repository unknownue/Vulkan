
#version 450 core
#extension GL_ARB_separate_shader_objects : enable

layout (location = 0) in vec2 inUV;
layout (location = 1) in vec4 inColor;

layout (location = 0) out vec4 outColor;

layout (binding = 0) uniform sampler2D font_glyphs;

void main() {

    vec4 color = vec4(inColor.xyz, inColor.w * texture(font_glyphs, inUV).r);

    if (color.a <= 0.3) {
        discard;
    }

    outColor = color;
}

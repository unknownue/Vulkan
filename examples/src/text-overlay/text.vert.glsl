
#version 450 core
#extension GL_ARB_separate_shader_objects : enable

layout (location = 0) in uint inCharacterId;
layout (location = 1) in vec2 inPos;
layout (location = 2) in vec2 inUV;
layout (location = 3) in vec4 inColor;

layout (location = 0) out vec2 outUV;
layout (location = 1) out vec4 outColor;
layout (location = 2) out uint outCharacterId;

//layout (binding = 0) uniform Uniforms {
//    mat4 screenSpaceNormalizeMat;
//} uniforms;

void main() {

    //gl_Position = vec4((mat3(uniforms.screenSpaceNormalizeMat) * vec3(inPos, 1.0)).xy, 0.0, 1.0);
    gl_Position = vec4(inPos, 0.0, 1.0);

    outUV = inUV;
    outColor = inColor;
    outCharacterId = inCharacterId;
}

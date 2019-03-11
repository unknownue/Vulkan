
#version 450

layout(location = 0) in vec3 inUV;

layout(location = 0) out vec4 outFragColor;

layout(set = 0, binding = 1) uniform sampler2DArray samplerArray;


void main() {

    outFragColor = texture(samplerArray, inUV);
}

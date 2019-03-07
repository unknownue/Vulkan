
#version 450

layout (location = 0) in vec3 inPos;
layout (location = 1) in vec2 inUV;

layout (location = 0) out vec2 outUV;

layout (set = 0, binding = 0) uniform UBOMatrices {
	mat4 projection;
	mat4 view;
	mat4 model;
} ubo;

layout (set = 0, binding = 1) uniform DynNode {
	mat4 transform;
} dyn_node;

out gl_PerVertex {
	vec4 gl_Position;
};

void main() {

	outUV = inUV;

    gl_Position = ubo.projection * ubo.view * ubo.model * dyn_node.transform * vec4(inPos.xyz, 1.0);
}

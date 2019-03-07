
#version 450

layout (location = 0) in vec3 inPos;
layout (location = 1) in vec3 inNormal;
layout (location = 2) in vec2 inUV;

layout (location = 0) out vec3 outNormal;
layout (location = 1) out vec3 outColor;
layout (location = 2) out vec2 outUV;

layout (set = 0, binding = 0) uniform UBOMatrices {
	mat4 projection;
	mat4 view;
	mat4 model;
} ubo;

layout (set = 0, binding = 1) uniform DynNode {
	mat4 transform;
} dyn_node;

layout (push_constant) uniform Material {
	vec4 base_color_factor;
	vec3 emissive_factor;
	float metallic_factor;
} material;

out gl_PerVertex {
	vec4 gl_Position;
};

void main() {

    outNormal = inNormal;
	outColor = material.base_color_factor.xyz;
	outUV    = inUV;

    gl_Position = ubo.projection * ubo.view * ubo.model * dyn_node.transform * vec4(inPos.xyz, 1.0);
}

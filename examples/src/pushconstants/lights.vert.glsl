
#version 450

#define lightCount 6

layout (location = 0) in vec3 inPos;
layout (location = 1) in vec3 inNormal;

layout (location = 0) out vec3 outNormal;
layout (location = 1) out vec4 outLightVec[lightCount];

layout (set = 0, binding = 0) uniform UBO {
	mat4 projection;
	mat4 view;
	mat4 model;
} ubo;

layout (set = 0, binding = 1) uniform DynNode {
	mat4 transform;
} dyn_node;

layout(push_constant) uniform PushConsts {
	vec4 lightPos[lightCount];
} pushConsts;

out gl_PerVertex {
	vec4 gl_Position;
};

void main() {

	outNormal = inNormal;

	gl_Position = ubo.projection * ubo.model * ubo.view * dyn_node.transform * vec4(inPos.xyz, 1.0);

	for (int i = 0; i < lightCount; ++i) {
		outLightVec[i].xyz = pushConsts.lightPos[i].xyz - inPos.xyz;
		// Store light radius in w
		outLightVec[i].w = pushConsts.lightPos[i].w;
	}
}

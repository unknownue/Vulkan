
#version 450

layout (location = 0) in vec3 inPos;
layout (location = 1) in vec3 inNormal;

layout (location = 0) out vec3 outNormal;
layout (location = 1) out vec3 outColor;
layout (location = 2) out vec3 outViewVec;
layout (location = 3) out vec3 outLightVec;

layout (set = 0, binding = 0) uniform UBO {
	mat4 projection;
	mat4 view;
	mat4 model;
	vec4 lightPos;
} ubo;

layout (set = 0, binding = 1) uniform NodeAttachments {
	mat4 transform;
} node_attachments;

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
	outColor  = material.base_color_factor.xyz;
	gl_Position = ubo.projection * ubo.view * ubo.model * node_attachments.transform * vec4(inPos.xyz, 1.0);
	
	vec4 pos = ubo.model * vec4(inPos, 1.0);
	outNormal = mat3(ubo.model) * inNormal;
	vec3 lPos = mat3(ubo.model) * ubo.lightPos.xyz;
	outLightVec = lPos - pos.xyz;
	outViewVec = -pos.xyz;
}

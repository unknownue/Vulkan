
#version 450

layout (location = 0) in vec3 inPos;
layout (location = 1) in vec3 inNormal;

layout (location = 0) out vec3 outColor;

layout (set = 0, binding = 0) uniform UBO {
	mat4 projection;
	mat4 view;
	mat4 model;
	mat4 y_correction;
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

    outColor = material.base_color_factor.xyz;

    gl_Position = ubo.y_correction * ubo.projection * ubo.view * ubo.model * node_attachments.transform * vec4(inPos.xyz, 1.0);
}


#version 450

layout (location = 0) in vec3 inPos;

layout (location = 0) out vec3 outUVW;

layout (set = 0, binding = 0) uniform UBO {
	mat4 projection;
	mat4 model;
} ubo;

layout (set = 0, binding = 1) uniform DynNode {
    mat4 transform;
} dyn_node;

out gl_PerVertex {
	vec4 gl_Position;
};

void main() {

	outUVW = inPos;
	outUVW.x *= -1.0;
	gl_Position = ubo.projection * ubo.model * dyn_node.transform * vec4(inPos.xyz, 1.0);
}

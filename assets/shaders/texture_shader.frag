#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec2 inTexCoord;
layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 1) uniform texture2D texture_;
layout(set = 0, binding = 2) uniform sampler sampler_;

void main() {
	outColor = texture(sampler2D(texture_, sampler_), inTexCoord);
}

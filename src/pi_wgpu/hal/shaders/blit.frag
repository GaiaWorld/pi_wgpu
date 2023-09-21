#version 450

precision highp float;

layout(location = 0) in vec2 vUv;

layout(location = 0) out vec4 o_Target;

// 纹理
layout(set = 0, binding = 0) uniform sampler samp;
layout(set = 0, binding = 1) uniform texture2D tex2d;

void main() {
    vec4 color = texture(sampler2D(tex2d, samp), vUv);

    o_Target = vec4(color.rgb, 1.0);
}
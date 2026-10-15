#version 450

layout(binding = 0) uniform texture2D u_tex;
layout(binding = 1) uniform sampler u_smp;

layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 o_color;

void main() {
    o_color = texture(sampler2D(u_tex, u_smp), v_uv);
}

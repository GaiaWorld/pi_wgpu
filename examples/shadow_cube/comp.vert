#version 450
layout(location = 0) in vec2 a_pos;
layout(location = 0) out vec2 v_uv;
void main() {
    // WebGPU 顶原点：场景纹理行 0=顶（ADJUST 后存储方向），V 由 NDC 反推须取 0.5-0.5y。
    v_uv = vec2(a_pos.x * 0.5 + 0.5, 0.5 - a_pos.y * 0.5);
    gl_Position = vec4(a_pos, 0.0, 1.0);
}

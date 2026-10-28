#version 450
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 o_color;
void main() {
    // 横向三色带：底=红 中=绿 顶=蓝（验证行序与内容）
    if (v_uv.y < 0.33) { o_color = vec4(1.0, 0.0, 0.0, 1.0); }
    else if (v_uv.y < 0.66) { o_color = vec4(0.0, 1.0, 0.0, 1.0); }
    else { o_color = vec4(0.0, 0.0, 1.0, 1.0); }
}

#version 450

layout(location = 0) in vec2 a_pos;
// 注：本示例 a_uv 仍为旧 GL 约定（v=0=底行）；产品代码按 documents/api.md §8.2-2「v=0=图像顶行」摆放，勿照抄。
layout(location = 1) in vec2 a_uv;

layout(location = 0) out vec2 v_uv;

void main() {
    gl_Position = vec4(a_pos, 0.0, 1.0);
    v_uv = a_uv;
}

#version 450
// 深度通道顶点：世界坐标直入光视投影（无 fragment stage）。
layout(std140, binding = 2) uniform Matrices {
    mat4 view_proj;
    mat4 light_vp;
    vec4 light_dir; // 指向光源（归一）
    vec4 params;    // x=texel(1/S), y=bias, z=0, w=0
};
layout(location = 0) in vec3 a_pos;
void main() {
    gl_Position = light_vp * vec4(a_pos, 1.0);
}

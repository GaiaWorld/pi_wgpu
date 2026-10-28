#version 450
layout(std140, binding = 2) uniform Matrices {
    mat4 view_proj;
    mat4 light_vp;
    vec4 light_dir; // 指向光源（归一）
    vec4 params;    // x=texel(1/S), y=bias, z=0, w=0
};
layout(location = 0) in vec3 a_pos;
layout(location = 1) in vec3 a_normal;
layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec4 v_light;
void main() {
    v_normal = a_normal;
    v_light = light_vp * vec4(a_pos, 1.0);
    gl_Position = view_proj * vec4(a_pos, 1.0);
}

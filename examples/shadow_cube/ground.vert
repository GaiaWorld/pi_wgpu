#version 450
layout(std140, binding = 2) uniform Matrices {
    mat4 view_proj;
    mat4 light_vp;
    vec4 light_dir;
    vec4 params;
};
layout(location = 0) in vec3 a_pos; // 世界 y=0 平面点 (x,0,z)
layout(location = 0) out vec3 v_world;
layout(location = 1) out vec4 v_light;
void main() {
    v_world = a_pos;
    v_light = light_vp * vec4(a_pos, 1.0);
    gl_Position = view_proj * vec4(a_pos, 1.0);
}
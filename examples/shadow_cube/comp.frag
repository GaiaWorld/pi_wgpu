#version 450
layout(binding = 0) uniform texture2D u_scene;
layout(binding = 1) uniform sampler u_scene_smp;
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 o_color;
void main() {
    o_color = texture(sampler2D(u_scene, u_scene_smp), v_uv);
}

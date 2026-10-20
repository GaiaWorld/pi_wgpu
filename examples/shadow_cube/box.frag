#version 450
layout(std140, binding = 2) uniform Matrices {
    mat4 view_proj;
    mat4 light_vp;
    vec4 light_dir;
    vec4 params;
};
layout(binding = 0) uniform texture2D u_shadow;
layout(binding = 1) uniform sampler u_shadow_smp;
layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec4 v_light;
layout(location = 0) out vec4 o_color;
float shadow_at(vec4 light_clip) {
    vec3 ndc = light_clip.xyz / max(light_clip.w, 0.0001);
    vec2 uv = ndc.xy * 0.5 + 0.5;
    float d = ndc.z * 0.5 + 0.5;
    float texel = params.x;
    float bias = params.y;
    float s = 0.0;
    for (int iy = 0; iy < 2; iy++) {
        for (int ix = 0; ix < 2; ix++) {
            vec2 o = vec2(float(ix), float(iy)) * texel;
            float c = texture(sampler2D(u_shadow, u_shadow_smp), uv + o).r;
            if (d - bias <= c) { s += 1.0; }
        }
    }
    float inside = 1.0;
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) { inside = 0.0; }
    return s * 0.25 * (1.0 - 0.65 * (1.0 - inside));
}
void main() {
    vec3 n = normalize(v_normal);
    float lambert = max(dot(n, normalize(light_dir.xyz)), 0.0);
    float sh = shadow_at(v_light);
    vec3 base = vec3(0.85, 0.45, 0.2);
    vec3 c = base * (0.22 + 0.78 * lambert) * sh;
    o_color = vec4(c, 1.0);
}
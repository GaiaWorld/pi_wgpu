#version 450

layout (location = 0) in vec2 position;
layout (location = 1) in vec2 uv;

out vec2 vUv;  // 输出到片段着色器的颜色

void main()
{
    gl_Position = vec4(position, 0.0, 1.0);

    vUv = uv;
}

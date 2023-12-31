#version 450

layout (location = 0) in vec2 position;
layout (location = 1) in vec4 color;

out vec4 fragColor;  // 输出到片段着色器的颜色

void main()
{
    gl_Position = vec4(position, 0.0, 1.0);

    fragColor = color;
}

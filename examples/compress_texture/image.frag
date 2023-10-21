#version 450

in vec2 vUv;

out vec4 outColor;

layout(set=0,binding=0)uniform sampler samp;
layout(set=0,binding=1)uniform texture2D tex2d;

void main(){
	outColor=vec4(texture(sampler2D(tex2d,samp),vUv), 0.0, 0.0, 1.0);
	// outColor = vec4(vUv.x, uv.y, 0.0, 1.0);
}
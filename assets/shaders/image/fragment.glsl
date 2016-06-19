#version 110

varying vec2 v_Texture;

uniform sampler2D texture;
uniform float alpha;

void main() {
	vec4 color = texture2D(texture, v_Texture);
	color.a = min(color.a, alpha);

	gl_FragColor = color;
}

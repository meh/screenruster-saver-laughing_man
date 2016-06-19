#version 110

uniform sampler2D texture;
varying vec2 v_Texture;

void main() {
	vec4 color = texture2D(texture, v_Texture);

	if (color.a == 0.5) {
		color.a = 0.0;
	}

	gl_FragColor = color;
}

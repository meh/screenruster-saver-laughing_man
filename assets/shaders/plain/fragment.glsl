#version 110

uniform sampler2D texture;
varying vec2 v_Texture;

void main() {
	gl_FragColor = texture2D(texture, v_Texture);
}

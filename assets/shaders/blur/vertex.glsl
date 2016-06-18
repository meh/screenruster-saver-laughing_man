#version 110

attribute vec2 position;
attribute vec2 texture;

varying vec2 v_Texture;

void main() {
	v_Texture = texture;
	gl_Position = vec4(position, 0.0, 1.0);
}

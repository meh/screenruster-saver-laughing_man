#version 110

uniform mat4 mvp;

attribute vec2 position;
attribute vec2 texture;

varying vec2 v_Texture;

void main() {
	gl_Position = mvp * vec4(position, 0.0, 1.0);
	v_Texture = texture;
}

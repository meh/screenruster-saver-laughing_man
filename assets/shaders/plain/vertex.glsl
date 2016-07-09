#version 110

attribute vec2 position;
attribute vec2 texture;

varying vec2 v_Texture;

uniform mat4 mvp;

void main() {
	gl_Position = mvp * vec4(position, 0.0, 1.0);
	v_Texture = texture;
}

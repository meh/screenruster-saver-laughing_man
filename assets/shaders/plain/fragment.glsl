uniform sampler2D texture;
varying vec2 v_Texture;

void main() {
	gl_FragColor = vec4(texture2D(texture, v_Texture).rgb, 1.0);
}

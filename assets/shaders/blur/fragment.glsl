#version 110

// The texture coordinates.
varying vec2 v_Texture;

// The texture to blur.
uniform sampler2D texture;

// The direction of the blur:
//
// (1.0, 0.0) -> x-axis blur
// (0.0, 1.0) -> y-axis blur
uniform vec2 direction;

// The blur radius.
uniform float radius;

// The resolution (width or height depending on direction).
uniform float resolution;

// Apply blurring, using a 9-tap filter with predefined gaussian weights.
void main() {
	// The amount to blur, i.e. how far off center to sample from:
	//
	// 1.0 -> blur by one pixel
	// 2.0 -> blur by two pixels, ev_Texture.
	float blur = radius / resolution;
	vec4  sum  = vec4(0.0);

	sum += texture2D(texture, vec2(v_Texture.x - 4.0 * blur * direction.x, v_Texture.y - 4.0 * blur * direction.y)) * 0.0162162162;
	sum += texture2D(texture, vec2(v_Texture.x - 3.0 * blur * direction.x, v_Texture.y - 3.0 * blur * direction.y)) * 0.0540540541;
	sum += texture2D(texture, vec2(v_Texture.x - 2.0 * blur * direction.x, v_Texture.y - 2.0 * blur * direction.y)) * 0.1216216216;
	sum += texture2D(texture, vec2(v_Texture.x - 1.0 * blur * direction.x, v_Texture.y - 1.0 * blur * direction.y)) * 0.1945945946;

	sum += texture2D(texture, vec2(v_Texture.x, v_Texture.y)) * 0.2270270270;

	sum += texture2D(texture, vec2(v_Texture.x + 1.0 * blur * direction.x, v_Texture.y + 1.0 * blur * direction.y)) * 0.1945945946;
	sum += texture2D(texture, vec2(v_Texture.x + 2.0 * blur * direction.x, v_Texture.y + 2.0 * blur * direction.y)) * 0.1216216216;
	sum += texture2D(texture, vec2(v_Texture.x + 3.0 * blur * direction.x, v_Texture.y + 3.0 * blur * direction.y)) * 0.0540540541;
	sum += texture2D(texture, vec2(v_Texture.x + 4.0 * blur * direction.x, v_Texture.y + 4.0 * blur * direction.y)) * 0.0162162162;

	gl_FragColor = vec4(sum.rgb, 1.0);
}

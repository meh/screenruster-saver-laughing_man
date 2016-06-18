use std::rc::Rc;

use api;
use api::gl::{self, Surface};

use {Config, Vertex};

pub struct Saver {
	config: Config,
	state:  api::State,
	gl:     Option<Graphics>,

	blur: f32,
}

unsafe impl Send for Saver { }

pub struct Graphics {
	context: Rc<gl::backend::Context>,
	width:   u32,
	height:  u32,

	screen:  Screen,
}

struct Screen {
	transient: (gl::texture::Texture2d, gl::texture::Texture2d),
	vertex:    gl::VertexBuffer<Vertex>,
	index:     gl::IndexBuffer<u16>,
	blur:      gl::Program,
	plain:     gl::Program,
}

impl Saver {
	pub fn new(config: Config) -> Saver {
		Saver {
			config: config,
			state:  Default::default(),
			gl:     None,
			blur:   0.0,
		}
	}
}

impl api::Saver for Saver {
	fn initialize(&mut self, context: Rc<gl::backend::Context>) {
		let (width, height) = context.get_framebuffer_dimensions();

		let screen = {
			let transient = (gl::texture::Texture2d::empty(&context, width, height).unwrap(),
			                 gl::texture::Texture2d::empty(&context, width, height).unwrap());

			let vertex = gl::VertexBuffer::new(&context, &[
				Vertex { position: [-1.0, -1.0], texture: [0.0, 0.0] },
				Vertex { position: [-1.0,  1.0], texture: [0.0, 1.0] },
				Vertex { position: [ 1.0,  1.0], texture: [1.0, 1.0] },
				Vertex { position: [ 1.0, -1.0], texture: [1.0, 0.0] },
			]).unwrap();

			let index = gl::IndexBuffer::new(&context, gl::index::PrimitiveType::TriangleStrip,
				&[1 as u16, 2, 0, 3]).unwrap();

			let blur = program!(&context,
				110 => {
					vertex:   include_str!("../assets/shaders/blur/vertex.glsl"),
					fragment: include_str!("../assets/shaders/blur/fragment.glsl"),
				},
			).unwrap();

			let plain = program!(&context,
				110 => {
					vertex:   include_str!("../assets/shaders/plain/vertex.glsl"),
					fragment: include_str!("../assets/shaders/plain/fragment.glsl"),
				},
			).unwrap();

			Screen {
				transient: transient,
				vertex:    vertex,
				index:     index,
				blur:      blur,
				plain:     plain,
			}
		};

		self.gl = Some(Graphics {
			context: context,
			width:   width,
			height:  height,

			screen: screen,
		});
	}

	// Go at 30 FPS.
	fn step(&self) -> f64 {
		0.3
	}

	fn begin(&mut self) {
		self.state = api::State::Begin;
	}

	fn end(&mut self) {
		self.state = api::State::End;
	}

	fn state(&self) -> api::State {
		self.state
	}

	fn update(&mut self) {
		match self.state {
			api::State::Begin => {
				if let Some(blur) = self.config.blur {
					if self.blur < blur.max {
						self.blur += blur.step;
					}
					else {
						self.state = api::State::Running;
					}
				}
			}

			api::State::Running => {
				// Move around the laughing man.
			}

			api::State::End => {
				if let Some(blur) = self.config.blur {
					if self.blur > 0.0 {
						self.blur -= blur.step;
					}
					else {
						self.state = api::State::None;
					}
				}
			}

			api::State::None => (),
		}
	}

	fn render(&self, target: &mut gl::Frame, screen: &gl::texture::Texture2d) {
		let gl = self.gl.as_ref().unwrap();

		// Blur the screen.
		if let Some(blur) = self.config.blur {
			let mut frame = (gl::framebuffer::SimpleFrameBuffer::new(&gl.context, &gl.screen.transient.0).unwrap(),
			                 gl::framebuffer::SimpleFrameBuffer::new(&gl.context, &gl.screen.transient.1).unwrap());

			// Draw screen to frame.
			{
				let uniforms = uniform! {
					texture: screen.sampled(),
				};

				frame.1.draw(&gl.screen.vertex, &gl.screen.index, &gl.screen.plain, &uniforms, &Default::default()).unwrap();
			}

			for _ in 0 .. blur.count {
				// Draw the screen to the texture with horizontal blur.
				{
					let uniforms = uniform! {
						texture: gl.screen.transient.1.sampled()
							.wrap_function(gl::uniforms::SamplerWrapFunction::Repeat),

						radius:     self.blur,
						resolution: gl.width as f32,
						direction:  (1.0, 0.0): (f32, f32),
					};

					frame.0.draw(&gl.screen.vertex, &gl.screen.index, &gl.screen.blur, &uniforms, &Default::default()).unwrap();
				}

				// Draw the texture to the screen with vertical blur.
				{
					let uniforms = uniform! {
						texture: gl.screen.transient.0.sampled()
							.wrap_function(gl::uniforms::SamplerWrapFunction::Repeat),

						radius:     self.blur,
						resolution: gl.height as f32,
						direction:  (0.0, 1.0): (f32, f32),
					};

					frame.1.draw(&gl.screen.vertex, &gl.screen.index, &gl.screen.blur, &uniforms, &Default::default()).unwrap();
				}
			}

			// Draw blurred texture to screen.
			{
				let uniforms = uniform! {
					texture: gl.screen.transient.1.sampled(),
				};

				target.draw(&gl.screen.vertex, &gl.screen.index, &gl.screen.plain, &uniforms, &Default::default()).unwrap();
			}
		}

		// Draw laughing man.
//		{
//			target.draw(&gl.screen.vertex, &gl.screen.index, &gl.screen.blur, &uniforms, &gl::DrawParameters {
//				blend: gl::Blend {
//					color: gl::BlendingFunction::Addition {
//						source:      gl::LinearBlendingFactor::SourceAlpha,
//						destination: gl::LinearBlendingFactor::OneMinusSourceAlpha
//					},
//
//					alpha: gl::BlendingFunction::Addition {
//						source:      gl::LinearBlendingFactor::SourceAlpha,
//						destination: gl::LinearBlendingFactor::OneMinusSourceAlpha
//					},
//
//					.. Default::default()
//				},
//
//				.. Default::default()
//			}).unwrap();
//
//		}
	}
}

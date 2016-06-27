use std::rc::Rc;

use screen;
use screen::json::JsonValue;
use screen::gl::{self, Surface};
use screen::image::GenericImage;

use {Config, Vertex, Scene};

pub struct Saver {
	config: Option<Config>,
	state:  screen::State,
	gl:     Option<Graphics>,

	dialog: bool,
	blur:   f32,
	man:    Option<Man>,
}

unsafe impl Send for Saver { }

#[derive(Copy, Clone, Debug)]
pub struct Man {
	x: u32,
	y: u32,

	alpha:    (f32, f32),
	rotation: (f32, u32),
	scale:    f32,
}

pub struct Graphics {
	context: Rc<gl::backend::Context>,
	width:   u32,
	height:  u32,

	scene:  Scene,
	screen: graphics::Screen,
	man:    graphics::Man,
}

mod graphics {
	use screen::gl;
	use Vertex;

	pub struct Screen {
		pub transient: (gl::texture::Texture2d, gl::texture::Texture2d),
		pub vertex:    gl::VertexBuffer<Vertex>,
		pub index:     gl::IndexBuffer<u16>,
		pub blur:      gl::Program,
		pub plain:     gl::Program,
	}

	pub struct Man {
		pub fixed:    Image,
		pub dynamic:  Image,
		pub complete: Image,
	}

	pub struct Image {
		pub texture: gl::texture::Texture2d,
		pub vertex:  gl::VertexBuffer<Vertex>,
		pub index:   gl::IndexBuffer<u16>,
		pub program: gl::Program,
	}
}

impl Saver {
	pub fn new() -> Saver {
		Saver {
			config: None,
			state:  Default::default(),
			gl:     None,

			dialog: false,
			blur:   0.0,
			man:    None,
		}
	}
}

impl screen::Saver for Saver {
	fn config(&mut self, config: JsonValue) {
		self.config = Some(Config::new(config));
	}

	fn initialize(&mut self, context: Rc<gl::backend::Context>) {
		let config          = self.config.as_ref().unwrap();
		let (width, height) = context.get_framebuffer_dimensions();

		let scene = Scene::new(width, height);

		let screen = {
			// The transient textures are needed to do alternated blurring.
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

			graphics::Screen {
				transient: transient,
				vertex:    vertex,
				index:     index,
				blur:      blur,
				plain:     plain,
			}
		};

		macro_rules! load {
			($path:expr) => ({
				let image   = screen::image::load_from_memory(include_bytes!($path)).unwrap();
				let size    = image.dimensions();
				let image   = gl::texture::RawImage2d::from_raw_rgba_reversed(image.to_rgba().into_raw(), size);
				let texture = gl::texture::Texture2d::with_mipmaps(&context, image, gl::texture::MipmapsOption::NoMipmap).unwrap();

				let vertex = gl::VertexBuffer::new(&context, &[
					Vertex { position: [-1.0, -1.0], texture: [0.0, 0.0] },
					Vertex { position: [-1.0,  1.0], texture: [0.0, 1.0] },
					Vertex { position: [ 1.0,  1.0], texture: [1.0, 1.0] },
					Vertex { position: [ 1.0, -1.0], texture: [1.0, 0.0] },
				]).unwrap();

				let index = gl::IndexBuffer::new(&context, gl::index::PrimitiveType::TriangleStrip,
					&[1 as u16, 2, 0, 3]).unwrap();

				let program = program!(&context,
					110 => {
						vertex:   include_str!("../assets/shaders/image/vertex.glsl"),
						fragment: include_str!("../assets/shaders/image/fragment.glsl"),
					},
				).unwrap();

				graphics::Image {
					texture: texture,
					vertex:  vertex,
					index:   index,
					program: program,
				}

			});
		}

		let man = graphics::Man {
			fixed:    load!("../assets/fixed.png"),
			dynamic:  load!("../assets/dynamic.png"),
			complete: load!("../assets/complete.png"),
		};

		// Initiate `Man` and calculate step from blur step.
		self.man = Some({
			let step = if let Some(blur) = config.blur {
				blur.step / blur.max
			}
			else {
				0.001
			};

			Man {
				x: width / 2,
				y: height / 2,

				alpha:    (0.0, step),
				rotation: (0.0, 0),
				scale:    config.man.scale,
			}
		});

		self.gl = Some(Graphics {
			context: context,
			width:   width,
			height:  height,

			scene:  scene,
			screen: screen,
			man:    man,
		});
	}

	fn begin(&mut self) {
		self.state = screen::State::Begin;
	}

	fn end(&mut self) {
		self.state = screen::State::End;
	}

	fn state(&self) -> screen::State {
		self.state
	}

	fn dialog(&mut self, active: bool) {
		self.dialog = active;
	}

	fn update(&mut self) {
		let config = self.config.as_ref().unwrap();
		let man    = self.man.as_mut().unwrap();

		match self.state {
			screen::State::Begin => {
				if let Some(blur) = config.blur {
					if self.blur < blur.max {
						self.blur   += blur.step;
						man.alpha.0 += man.alpha.1;
					}
					else {
						self.state  = screen::State::Running;
						man.alpha.0 = 1.0;
					}
				}
			}

			screen::State::Running => {
				if let Some(step) = config.man.rotate {
					man.rotation.1 += 1;
					man.rotation.0  = man.rotation.1 as f32 * step;

					if man.rotation.0 > 360.0 {
						man.rotation = (0.0, 0);
					}
				}
			}

			screen::State::End => {
				if let Some(blur) = config.blur {
					if self.blur > 0.0 {
						self.blur   -= blur.step;
						man.alpha.0 -= man.alpha.1;
					}
					else {
						self.state  = screen::State::None;
						man.alpha.0 = 0.0;
					}
				}
			}

			screen::State::None => (),
		}
	}

	fn render<S: Surface>(&self, target: &mut S, screen: &gl::texture::Texture2d) {
		let gl     = self.gl.as_ref().unwrap();
		let config = self.config.as_ref().unwrap();
		let man    = self.man.as_ref().unwrap();

		// Blur the screen.
		if let Some(blur) = config.blur {
			let mut frame = (gl::framebuffer::SimpleFrameBuffer::new(&gl.context, &gl.screen.transient.0).unwrap(),
			                 gl::framebuffer::SimpleFrameBuffer::new(&gl.context, &gl.screen.transient.1).unwrap());

			// Draw screen to frame.
			{
				let uniforms = uniform! {
					texture: screen.sampled(),
				};

				frame.1.draw(&gl.screen.vertex, &gl.screen.index, &gl.screen.plain, &uniforms, &Default::default()).unwrap();
			}

			// Repeat the blur to obtain a better effect.
			for _ in 0 .. blur.count {
				// Blur horizontally.
				{
					let uniforms = uniform! {
						texture: gl.screen.transient.1.sampled()
							.wrap_function(gl::uniforms::SamplerWrapFunction::Clamp),

						radius:     self.blur,
						resolution: gl.width as f32,
						direction:  (1.0, 0.0): (f32, f32),
					};

					frame.0.draw(&gl.screen.vertex, &gl.screen.index, &gl.screen.blur, &uniforms, &Default::default()).unwrap();
				}

				// Blur vertically.
				{
					let uniforms = uniform! {
						texture: gl.screen.transient.0.sampled()
							.wrap_function(gl::uniforms::SamplerWrapFunction::Clamp),

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
		
		// If the dialog is not open.
		if !self.dialog {
			let mvp = gl.scene.to_matrix()
				* gl.scene.position(man.x, man.y)
				* gl.scene.scale(man.scale);

			// If we're in rotation mode, compose the two images.
			if man.alpha.0 == 1.0 {
				// Draw dynamic image.
				{
					let mvp = mvp * gl.scene.rotate(man.rotation.0);

					let uniforms = uniform! {
						mvp:     *mvp.as_ref(),
						alpha:   man.alpha.0,
						texture: gl.man.dynamic.texture.sampled()
							.minify_filter(gl::uniforms::MinifySamplerFilter::Linear)
							.magnify_filter(gl::uniforms::MagnifySamplerFilter::Linear),
					};

					target.draw(&gl.man.dynamic.vertex, &gl.man.dynamic.index, &gl.man.dynamic.program, &uniforms, &gl::DrawParameters {
						blend: gl::Blend {
							color: gl::BlendingFunction::Addition {
								source:      gl::LinearBlendingFactor::SourceAlpha,
								destination: gl::LinearBlendingFactor::OneMinusSourceAlpha
							},

							alpha: gl::BlendingFunction::Addition {
								source:      gl::LinearBlendingFactor::SourceAlpha,
								destination: gl::LinearBlendingFactor::OneMinusSourceAlpha
							},

							.. Default::default()
						},

						.. Default::default()
					}).unwrap();
				}

				{
					let uniforms = uniform! {
						mvp:     *mvp.as_ref(),
						alpha:   man.alpha.0,
						texture: gl.man.fixed.texture.sampled()
							.minify_filter(gl::uniforms::MinifySamplerFilter::Linear)
							.magnify_filter(gl::uniforms::MagnifySamplerFilter::Linear),
					};

					target.draw(&gl.man.fixed.vertex, &gl.man.fixed.index, &gl.man.fixed.program, &uniforms, &gl::DrawParameters {
						blend: gl::Blend {
							color: gl::BlendingFunction::Addition {
								source:      gl::LinearBlendingFactor::SourceAlpha,
								destination: gl::LinearBlendingFactor::OneMinusSourceAlpha
							},

							alpha: gl::BlendingFunction::Addition {
								source:      gl::LinearBlendingFactor::SourceAlpha,
								destination: gl::LinearBlendingFactor::OneMinusSourceAlpha
							},

							.. Default::default()
						},

						.. Default::default()
					}).unwrap();
				}
			}
			else {
				let uniforms = uniform! {
					mvp:     *mvp.as_ref(),
					alpha:   man.alpha.0,
					texture: gl.man.complete.texture.sampled()
						.minify_filter(gl::uniforms::MinifySamplerFilter::Linear)
						.magnify_filter(gl::uniforms::MagnifySamplerFilter::Linear),
				};

				target.draw(&gl.man.fixed.vertex, &gl.man.fixed.index, &gl.man.fixed.program, &uniforms, &gl::DrawParameters {
					blend: gl::Blend {
						color: gl::BlendingFunction::Addition {
							source:      gl::LinearBlendingFactor::SourceAlpha,
							destination: gl::LinearBlendingFactor::OneMinusSourceAlpha
						},

						alpha: gl::BlendingFunction::Addition {
							source:      gl::LinearBlendingFactor::SourceAlpha,
							destination: gl::LinearBlendingFactor::OneMinusSourceAlpha
						},

						.. Default::default()
					},

					.. Default::default()
				}).unwrap();
			}
		}
	}
}

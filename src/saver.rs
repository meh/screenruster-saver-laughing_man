use std::rc::Rc;

use api;
use api::gl::{self, Surface};
use api::image::GenericImage;

use {Config, Vertex, Scene};

pub struct Saver {
	config: Config,
	state:  api::State,
	gl:     Option<Graphics>,

	blur:  f32,
	image: Option<Image>,
}

unsafe impl Send for Saver { }

#[derive(Copy, Clone, Debug)]
pub struct Image {
	x: u32,
	y: u32,

	rotation: f32,
	depth:    f32,
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
	use api::gl;
	use Vertex;

	pub struct Screen {
		pub transient: (gl::texture::Texture2d, gl::texture::Texture2d),
		pub vertex:    gl::VertexBuffer<Vertex>,
		pub index:     gl::IndexBuffer<u16>,
		pub blur:      gl::Program,
		pub plain:     gl::Program,
	}

	pub struct Man {
		pub fixed:   Image,
		pub dynamic: Image,
	}

	pub struct Image {
		pub texture: gl::texture::Texture2d,
		pub vertex:  gl::VertexBuffer<Vertex>,
		pub index:   gl::IndexBuffer<u16>,
		pub program: gl::Program,
	}
}

impl Saver {
	pub fn new(config: Config) -> Saver {
		Saver {
			config: config,
			state:  Default::default(),
			gl:     None,

			blur:  0.0,
			image: None,
		}
	}
}

impl api::Saver for Saver {
	fn initialize(&mut self, context: Rc<gl::backend::Context>) {
		let (width, height) = context.get_framebuffer_dimensions();

		let scene = Scene::new(width, height);

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

			graphics::Screen {
				transient: transient,
				vertex:    vertex,
				index:     index,
				blur:      blur,
				plain:     plain,
			}
		};

		let man = {
			let fixed = {
				let image   = api::image::load_from_memory(include_bytes!("../assets/fixed.png")).unwrap();
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
			};

			let dynamic = {
				let image   = api::image::load_from_memory(include_bytes!("../assets/dynamic.png")).unwrap();
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
			};

			graphics::Man {
				fixed:   fixed,
				dynamic: dynamic,
			}
		};

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
		self.state = api::State::Begin;
	}

	fn end(&mut self) {
		self.state = api::State::End;
	}

	fn state(&self) -> api::State {
		self.state
	}

	fn update(&mut self) {
		let gl = self.gl.as_ref().unwrap();

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
				if self.image.is_none() {
					self.image = Some(Image {
						x: gl.width / 2,
						y: gl.height / 2,

						depth:    0.0,
						rotation: 0.0,
					});
				}
				else {
					let image = self.image.as_mut().unwrap();

					if image.depth < self.config.image.depth {
						image.depth += 0.001;
					}

					if let Some(step) = self.config.image.rotate {
						image.rotation += step;

						if image.rotation > 360.0 {
							image.rotation = image.rotation - 360.0;
						}
					}
				}
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

		if let Some(image) = self.image {
			let mvp = gl.scene.to_matrix()
				* gl.scene.position(image.x, image.y)
				* gl.scene.depth(image.depth);

			// Draw dynamic image.
			{
				let mvp = mvp * gl.scene.rotate(image.rotation);

				let uniforms = uniform! {
					mvp:     *mvp.as_ref(),
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
	}
}

// Copyleft (ↄ) meh. <meh@schizofreni.co> | http://meh.schizofreni.co
//
// This file is part of screenruster.
//
// screenruster is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// screenruster is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with screenruster.  If not, see <http://www.gnu.org/licenses/>.

use std::rc::Rc;

use screen::{self, Password};
use screen::json::JsonValue;
use screen::gl::{self, Surface, program, uniform};

use crate::{Config, Vertex, Scene};

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
	rotation: (f32, u32, bool),
	scale:    f32,
	hue:      f32,
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
	use crate::Vertex;

	pub struct Screen {
		pub transient: (gl::texture::Texture2d, gl::texture::Texture2d),
		pub vertex:    gl::VertexBuffer<Vertex>,
		pub index:     gl::IndexBuffer<u16>,
		pub blur:      gl::Program,
		pub plain:     gl::Program,
	}

	pub struct Man {
		pub composite: gl::texture::Texture2d,
		pub vertex:    gl::VertexBuffer<Vertex>,
		pub index:     gl::IndexBuffer<u16>,
		pub program:   gl::Program,

		pub fixed:   Image,
		pub dynamic: Image,
	}

	pub struct Image {
		pub texture: gl::texture::Texture2d,
		pub width:   u32,
		pub height:  u32,
		pub vertex:  gl::VertexBuffer<Vertex>,
		pub index:   gl::IndexBuffer<u16>,
		pub program: gl::Program,
	}
}

impl Default for Saver {
	fn default() -> Saver {
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
				let image   = screen::picto::read::from_memory::<screen::picto::color::Rgba, u8, _>(&include_bytes!($path)[..]).unwrap();
				let size    = image.dimensions();
				let image   = gl::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), size);
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
						vertex:   include_str!("../assets/shaders/plain/vertex.glsl"),
						fragment: include_str!("../assets/shaders/plain/fragment.glsl"),
					},
				).unwrap();

				graphics::Image {
					texture: texture,
					width:   size.0,
					height:  size.1,

					vertex:  vertex,
					index:   index,
					program: program,
				}
			});
		}

		let man = {
			let fixed   = load!("../assets/fixed.png");
			let dynamic = load!("../assets/dynamic.png");

			let composite = gl::texture::Texture2d::empty(&context, fixed.width, fixed.height).unwrap();

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
					vertex:   include_str!("../assets/shaders/composite/vertex.glsl"),
					fragment: include_str!("../assets/shaders/composite/fragment.glsl"),
				},
			).unwrap();

			graphics::Man {
				composite: composite,
				vertex:    vertex,
				index:     index,
				program:   program,

				fixed:   fixed,
				dynamic: dynamic,
			}
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
				rotation: (0.0, 0, true),
				scale:    config.man.scale,
				hue:      0.0,
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

	fn start(&mut self) {
		self.state = screen::State::Begin;
	}

	fn stop(&mut self) {
		self.state = screen::State::End;
	}

	fn state(&self) -> screen::State {
		self.state
	}

	fn password(&mut self, password: Password) {
		if let Some(man) = self.man.as_mut() {
			match password {
				Password::Insert => {
					man.rotation.1 += 23;
				}

				Password::Delete => {
					man.rotation.1 -= 23;
				}

				Password::Reset => {
					man.rotation.1 = 0;
				}

				Password::Check => {
					man.rotation.2 = false;
				}

				Password::Success => {
					man.rotation.2 = true;
					man.hue        = -80.0;
				}

				Password::Failure => {
					man.rotation.2 = true;
					man.hue        = 150.0;
				}
			}
		}
	}

	fn update(&mut self) {
		let config = self.config.as_ref().unwrap();
		let man    = self.man.as_mut().unwrap();

		if let Some(step) = config.man.rotate {
			if man.rotation.2 {
				man.rotation.1 += 1;
				man.rotation.0  = man.rotation.1 as f32 * step;

				if man.rotation.0 > 360.0 {
					man.rotation.0 = 0.0;
					man.rotation.1 = 0;
				}
			}
		}

		match self.state {
			screen::State::Running |
			screen::State::None => (),

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
		}
	}

	fn render<S: Surface>(&self, target: &mut S, screen: &gl::texture::Texture2d) {
		let gl     = self.gl.as_ref().unwrap();
		let config = self.config.as_ref().unwrap();
		let man    = self.man.as_ref().unwrap();

		// Blur the screen.
		if let Some(blur) = config.blur {
			if self.state != screen::State::Running {
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
			}

			// Draw blurred texture to screen.
			{
				let uniforms = uniform! {
					mvp:     gl.scene.none().into(): [[f32; 4]; 4],
					texture: gl.screen.transient.1.sampled(),
				};

				target.draw(&gl.screen.vertex, &gl.screen.index, &gl.screen.plain, &uniforms, &Default::default()).unwrap();
			}
		}
		
		// If the dialog is not open.
		if !self.dialog {
			let mut frame = gl::framebuffer::SimpleFrameBuffer::new(&gl.context, &gl.man.composite).unwrap();
			frame.clear_color(0.0, 0.0, 0.0, 0.0);

			// Draw dynamic image.
			{
				let uniforms = uniform! {
					mvp:     gl.scene.rotate(man.rotation.0).into(): [[f32; 4]; 4],
					texture: gl.man.dynamic.texture.sampled()
						.minify_filter(gl::uniforms::MinifySamplerFilter::Linear)
						.magnify_filter(gl::uniforms::MagnifySamplerFilter::Linear),
				};

				frame.draw(&gl.man.dynamic.vertex, &gl.man.dynamic.index, &gl.man.dynamic.program, &uniforms, &gl::DrawParameters {
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

			// Draw fixed image.
			{
				let uniforms = uniform! {
					mvp:     gl.scene.none().into(): [[f32; 4]; 4],
					texture: gl.man.fixed.texture.sampled()
						.minify_filter(gl::uniforms::MinifySamplerFilter::Linear)
						.magnify_filter(gl::uniforms::MagnifySamplerFilter::Linear),
				};

				frame.draw(&gl.man.fixed.vertex, &gl.man.fixed.index, &gl.man.fixed.program, &uniforms, &gl::DrawParameters {
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

			// Draw composition to target with alpha and shifted hue.
			{
				let mvp = gl.scene.to_matrix()
					* gl.scene.position(man.x, man.y)
					* gl.scene.scale(man.scale);

				let uniforms = uniform! {
					mvp:     mvp.into(): [[f32; 4]; 4],
					alpha:   man.alpha.0,
					hue:     man.hue,
					texture: gl.man.composite.sampled()
						.minify_filter(gl::uniforms::MinifySamplerFilter::Linear)
						.magnify_filter(gl::uniforms::MagnifySamplerFilter::Linear),
				};

				target.draw(&gl.man.vertex, &gl.man.index, &gl.man.program, &uniforms, &gl::DrawParameters {
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

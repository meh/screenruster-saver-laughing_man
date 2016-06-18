#[macro_use]
extern crate api;

use std::rc::Rc;

use api::gl::{self, Surface};
use api::config;

pub struct Saver {
	config: config::Table,
	state:  api::State,
	gl:     Option<Graphics>
}

unsafe impl Send for Saver { }

pub struct Graphics {
	vertex:  gl::VertexBuffer<Vertex>,
	index:   gl::IndexBuffer<u16>,
	program: gl::Program,
}

#[derive(Copy, Clone)]
struct Vertex {
	position: [f32; 2],
	texture:  [f32; 2],
}

implement_vertex!(Vertex, position, texture);

impl Saver {
	pub fn new(config: config::Table) -> Box<Saver> {
		Box::new(Saver {
			config: config,
			state:  Default::default(),
			gl:     None,
		})
	}
}

impl api::Saver for Saver {
	fn initialize(&mut self, context: Rc<gl::backend::Context>) {
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
				vertex: "
					#version 110

					uniform mat4 matrix;

					attribute vec2 position;
					attribute vec2 texture;

					varying vec2 v_texture;

					void main() {
						gl_Position = matrix * vec4(position, 0.0, 1.0);
						v_texture = texture;
					}
				",

				fragment: "
					#version 110
					uniform sampler2D screen;
					varying vec2 v_texture;

					void main() {
						gl_FragColor = vec4(texture2D(screen, v_texture).rgb, 0.4);
					}
				",
			},
		).unwrap();

		self.gl = Some(Graphics {
			vertex:  vertex,
			index:   index,
			program: program,
		});
	}

	fn begin(&mut self) {
		self.state = api::State::Running;
	}

	fn end(&mut self) {
		self.state = api::State::None;
	}

	fn state(&self) -> api::State {
		self.state
	}

	fn render(&self, target: &mut gl::Frame, screen: &gl::texture::Texture2d) {
		let gl = self.gl.as_ref().unwrap();

		let uniforms = uniform! {
			matrix: [
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 1.0, 0.0, 0.0],
				[0.0, 0.0, 1.0, 0.0],
				[0.0, 0.0, 0.0, 1.0f32]
			],

			screen: screen,
		};

		target.draw(&gl.vertex, &gl.index, &gl.program, &uniforms, &gl::DrawParameters {
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

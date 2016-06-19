#![feature(type_ascription)]

#[macro_use]
extern crate api;

extern crate nalgebra as na;

mod config;
pub use config::Config;

mod saver;
pub use saver::Saver;

mod scene;
pub use scene::Scene;

#[derive(Copy, Clone)]
pub struct Vertex {
	position: [f32; 2],
	texture:  [f32; 2],
}

implement_vertex!(Vertex, position, texture);

pub fn new(config: api::config::Table) -> Box<saver::Saver> {
	Box::new(Saver::new(Config::new(config)))
}

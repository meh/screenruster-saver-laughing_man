#![feature(type_ascription)]

#[macro_use]
extern crate api;

mod saver;
pub use saver::Saver;

mod config;
pub use config::Config;

#[derive(Copy, Clone)]
pub struct Vertex {
	position: [f32; 2],
	texture:  [f32; 2],
}

implement_vertex!(Vertex, position, texture);

pub fn new(config: api::config::Table) -> Box<saver::Saver> {
	Box::new(Saver::new(Config::new(config)))
}

use api::config;

#[derive(Copy, Clone, Debug)]
pub struct Config {
	pub blur:  Option<Blur>,
	pub image: Image,
}

impl Default for Config {
	fn default() -> Config {
		Config {
			blur:  Some(Default::default()),
			image: Default::default(),
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub struct Blur {
	pub max:   f32,
	pub step:  f32,
	pub count: usize,
}

impl Default for Blur {
	fn default() -> Blur {
		Blur {
			max:   1.2,
			step:  0.00001,
			count: 4,
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub struct Image {
	pub rotate: Option<f32>,
	pub depth:  f32,
}

impl Default for Image {
	fn default() -> Image {
		Image {
			rotate: Some(0.2),
			depth:  200.0,
		}
	}
}

impl Config {
	pub fn new(table: config::Table) -> Config {
		let mut config = Config::default();

		match table.get("blur") {
			Some(&config::Value::Boolean(false)) => {
				config.blur = None
			}

			Some(&config::Value::Table(ref table)) => {
				let mut blur = Blur::default();

				if let Some(value) = table.get("max").and_then(|v| v.as_float()) {
					blur.max = value as f32;
				}

				if let Some(value) = table.get("step").and_then(|v| v.as_float()) {
					blur.step = value as f32;
				}

				if let Some(value) = table.get("count").and_then(|v| v.as_integer()) {
					blur.count = value as usize;
				}

				config.blur = Some(blur);
			}

			_ => ()
		}

		if let Some(table) = table.get("image").and_then(|v| v.as_table()) {
			let mut image = Image::default();

			if let Some(value) = table.get("depth").and_then(|v| v.as_float()) {
				image.depth = value as f32;
			}

			match table.get("rotate") {
				Some(&config::Value::Boolean(false)) => {
					image.rotate = None
				}

				Some(&config::Value::Float(value)) => {
					image.rotate = Some(value as f32);
				}

				_ => ()
			}

			config.image = image;
		}

		config
	}
}

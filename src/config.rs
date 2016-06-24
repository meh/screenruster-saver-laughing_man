use screen::json::JsonValue;

#[derive(Copy, Clone, Debug)]
pub struct Config {
	pub blur: Option<Blur>,
	pub man:  Man,
}

impl Default for Config {
	fn default() -> Config {
		Config {
			blur: Some(Default::default()),
			man:  Default::default(),
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
pub struct Man {
	pub rotate: Option<f32>,
	pub scale:  f32,
}

impl Default for Man {
	fn default() -> Man {
		Man {
			rotate: Some(0.000005),
			scale:  400.0,
		}
	}
}

impl Config {
	pub fn new(table: JsonValue) -> Config {
		let mut config = Config::default();

		// Blur.
		{
			if let Some(false) = table["blur"].as_bool() {
				config.blur = None
			}
			else {
				let mut blur = Blur::default();

				if let Some(value) = table["blur"]["max"].as_f32() {
					blur.max = value;
				}

				if let Some(value) = table["blur"]["step"].as_f32() {
					blur.step = value;
				}

				if let Some(value) = table["blur"]["count"].as_usize() {
					blur.count = value;
				}

				config.blur = Some(blur);
			}
		}

		// Man.
		{
			if let Some(value) = table["man"]["scale"].as_f32() {
				config.man.scale = value;
			}

			if let Some(false) = table["man"]["rotate"].as_bool() {
				config.man.rotate = None;
			}
			else if let Some(value) = table["man"]["rotate"].as_f32() {
				config.man.rotate = Some(value);
			}
		}

		config
	}
}

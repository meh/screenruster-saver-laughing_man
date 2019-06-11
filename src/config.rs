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
			step:  0.01,
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

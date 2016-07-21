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

use std::cmp;
use na;

pub struct Scene {
	width:  u32,
	height: u32,

	projection: na::OrthographicMatrix3<f32>,
}

impl Scene {
	pub fn new(width: u32, height: u32) -> Scene {
		let w = width as f32;
		let h = height as f32;

		Scene {
			width:  width,
			height: height,

			projection: na::OrthographicMatrix3::new(-w / 2.0, w / 2.0, -h / 2.0, h / 2.0, 0.1, 1000.0),
		}
	}

	pub fn to_matrix(&self) -> na::Matrix4<f32> {
		self.projection.to_matrix()
	}

	pub fn none(&self) -> na::Matrix4<f32> {
		na::new_identity(4)
	}

	pub fn position(&self, x: u32, y: u32) -> na::Matrix4<f32> {
		let x = x as f32;
		let y = y as f32;
		let w = self.width as f32;
		let h = self.height as f32;

		na::to_homogeneous(&na::Isometry3::new(na::Vector3::new(
			if x > w / 2.0 {
				-((w / 2.0) - x)
			}
			else {
				x - w / 2.0
			},

			-if y > h / 2.0 {
				-((h / 2.0) - y)
			}
			else {
				y - h / 2.0
			}, -500.0), na::zero()))
	}

	pub fn rotate(&self, deg: f32) -> na::Matrix4<f32> {
		na::to_homogeneous(&na::Rotation3::new_with_euler_angles(0.0, 0.0, deg))
	}

	pub fn scale(&self, size: f32) -> na::Matrix4<f32> {
		let size = size * (cmp::min(self.width, self.height) as f32 / 1080.0);

		na::Matrix4::new(size, 0.0,  0.0,    0.0,
		                 0.0,  size, 0.0,    0.0,
		                 0.0,  0.0,  size, 0.0,
		                 0.0,  0.0,  0.0,    1.0)
	}
}

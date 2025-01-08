use std::f32::consts::TAU;

use super::WindowingFn;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HannWindow;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RectangleWindow {
	rect_width: usize,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct IdentityWindow;

impl HannWindow {
	#[must_use]
	pub fn new() -> Self {
		Self
	}
}

impl RectangleWindow {
	#[must_use]
	pub fn new(rect_width: usize) -> Self {
		Self { rect_width }
	}
}

impl IdentityWindow {
	#[must_use]
	pub fn new() -> Self {
		Self
	}
}

impl WindowingFn for HannWindow {
	fn ratio_at(&mut self, index: usize, n_of_samples: usize) -> f32 {
		#[allow(clippy::cast_precision_loss)]
		return 0.5 * (1. - f32::cos((TAU * (index as f32)) / (n_of_samples - 1) as f32));
	}
}

impl WindowingFn for RectangleWindow {
	fn ratio_at(&mut self, index: usize, n_of_samples: usize) -> f32 {
		let rect_width = self.rect_width.min(n_of_samples);

		let offset = (n_of_samples - rect_width) / 2;

		if index < offset || index > n_of_samples - 1 - offset {
			0.
		} else {
			1.
		}
	}
}

impl WindowingFn for IdentityWindow {
	fn ratio_at(&mut self, _index: usize, _n_of_samples: usize) -> f32 {
		1.
	}
}

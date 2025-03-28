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
	pub const fn new() -> Self {
		Self
	}
}

impl RectangleWindow {
	#[must_use]
	pub const fn new(rect_width: usize) -> Self {
		Self { rect_width }
	}
}

impl IdentityWindow {
	#[must_use]
	pub const fn new() -> Self {
		Self
	}
}

impl WindowingFn for HannWindow {
	#[inline]
	fn ratio_at(&self, sample_idx: usize, n_of_samples: usize) -> f32 {
		#[allow(clippy::cast_precision_loss)]
		return 0.5 * (1. - f32::cos((TAU * (sample_idx as f32)) / (n_of_samples - 1) as f32));
	}
}

impl WindowingFn for RectangleWindow {
	#[inline]
	fn ratio_at(&self, sample_idx: usize, n_of_samples: usize) -> f32 {
		let rect_width = self.rect_width.min(n_of_samples);

		let offset = (n_of_samples - rect_width) / 2;

		if sample_idx < offset || sample_idx > n_of_samples - 1 - offset {
			0.
		} else {
			1.
		}
	}
}

impl WindowingFn for IdentityWindow {
	#[inline]
	fn ratio_at(&self, _sample_idx: usize, _n_of_samples: usize) -> f32 {
		1.
	}
}

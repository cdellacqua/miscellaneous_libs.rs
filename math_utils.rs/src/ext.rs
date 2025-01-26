#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]

use std::time::Duration;

pub trait DivisibleByUsize {
	#[must_use]
	fn div(self, rhs: usize) -> Self;
}

impl DivisibleByUsize for f32 {
	fn div(self, rhs: usize) -> f32 {
		self / rhs as f32
	}
}

impl DivisibleByUsize for f64 {
	fn div(self, rhs: usize) -> f64 {
		self / rhs as f64
	}
}

impl DivisibleByUsize for Duration {
	fn div(self, rhs: usize) -> Duration {
		self / rhs as u32
	}
}

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

pub trait Average {
	#[must_use]
	fn avg(self, rhs: Self) -> Self;
}

impl Average for f32 {
	fn avg(self, rhs: Self) -> Self {
		(self + rhs) / 2.
	}
}

impl Average for f64 {
	fn avg(self, rhs: Self) -> Self {
		(self + rhs) / 2.
	}
}

macro_rules! impl_avg_for_integer {
	($t:ty) => {
		impl Average for $t {
			fn avg(self, rhs: Self) -> Self {
				(self + rhs) / 2
			}
		}
	};
	($t:ty, $($others:ty),+) => {
		impl_avg_for_integer!($t);
		impl_avg_for_integer!($($others),+);
	};
}

impl_avg_for_integer!(u8, u16, u32, u64, u128, isize, i8, i16, i32, i64, i128, usize);

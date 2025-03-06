#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]

use std::time::Duration;

pub trait MultiplyByUsize {
	#[must_use]
	fn mul_usize(self, rhs: usize) -> Self;
}

macro_rules! impl_mul_for_float {
	($t:ty) => {
		impl MultiplyByUsize for $t {
			fn mul_usize(self, rhs: usize) -> Self {
				self * rhs as Self
			}
		}
	};
	($t:ty, $($others:ty),+) => {
		impl_mul_for_float!($t);
		impl_mul_for_float!($($others),+);
	};
}

impl_mul_for_float!(f32, f64);

pub trait DivisibleByUsize {
	#[must_use]
	fn div_usize(self, rhs: usize) -> Self;
}

macro_rules! impl_div_for_float {
	($t:ty) => {
		impl DivisibleByUsize for $t {
			fn div_usize(self, rhs: usize) -> Self {
				self / rhs as Self
			}
		}
	};
	($t:ty, $($others:ty),+) => {
		impl_div_for_float!($t);
		impl_div_for_float!($($others),+);
	};
}

impl_div_for_float!(f32, f64);


impl DivisibleByUsize for Duration {
	fn div_usize(self, rhs: usize) -> Duration {
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

pub trait RoundToUsize {
	#[must_use]
	fn round_usize(self) -> usize;
}

macro_rules! impl_round_for_float {
	($t:ty) => {
		#[allow(clippy::cast_sign_loss)]
		impl RoundToUsize for $t {
			fn round_usize(self) -> usize {
				(self + 0.5) as usize
			}
		}
	};
	($t:ty, $($others:ty),+) => {
		impl_round_for_float!($t);
		impl_round_for_float!($($others),+);
	};
}

impl_round_for_float!(f32, f64);

pub trait TruncToUsize {
	#[must_use]
	fn trunc_usize(self) -> usize;
}

macro_rules! impl_trunc_for_float {
	($t:ty) => {
		#[allow(clippy::cast_sign_loss)]
		impl TruncToUsize for $t {
			fn trunc_usize(self) -> usize {
				self as usize
			}
		}
	};
	($t:ty, $($others:ty),+) => {
		impl_trunc_for_float!($t);
		impl_trunc_for_float!($($others),+);
	};
}

impl_trunc_for_float!(f32, f64);

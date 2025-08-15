#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]

use std::time::Duration;

pub trait MultiplyByUsize {
	#[must_use]
	fn mul_usize(self, rhs: usize) -> Self;
}

macro_rules! impl_mul_for {
	($t:ty) => {
		impl MultiplyByUsize for $t {
			fn mul_usize(self, rhs: usize) -> Self {
				self * rhs as Self
			}
		}
	};
	($t:ty, $($others:ty),+) => {
		impl_mul_for!($t);
		impl_mul_for!($($others),+);
	};
}

impl_mul_for!(f32, f64);

pub trait DivisibleByUsize {
	#[must_use]
	fn div_usize(self, rhs: usize) -> Self;
}

macro_rules! impl_div_for {
	($t:ty) => {
		impl DivisibleByUsize for $t {
			fn div_usize(self, rhs: usize) -> Self {
				self / rhs as Self
			}
		}
	};
	($t:ty, $($others:ty),+) => {
		impl_div_for!($t);
		impl_div_for!($($others),+);
	};
}

impl_div_for!(f32, f64);

impl DivisibleByUsize for Duration {
	fn div_usize(self, rhs: usize) -> Duration {
		self / rhs as u32
	}
}

pub trait Average {
	#[must_use]
	fn avg(self, rhs: Self) -> Self;
}
macro_rules! impl_avg_for {
	($t:ty) => {
		impl Average for $t {
			fn avg(self, rhs: Self) -> Self {
				Self::midpoint(self, rhs)
			}
		}
	};
	($t:ty, $($others:ty),+) => {
		impl_avg_for!($t);
		impl_avg_for!($($others),+);
	};
}

impl_avg_for!(u8, u16, u32, u64, u128, isize, i8, i16, i32, i64, i128, usize/* , f16 */, f32, f64/* , f128 */);

pub trait RoundToUsize {
	#[must_use]
	fn round_usize(self) -> usize;
}

macro_rules! impl_round_for {
	($t:ty) => {
		#[allow(clippy::cast_sign_loss)]
		impl RoundToUsize for $t {
			fn round_usize(self) -> usize {
				(self + 0.5) as usize
			}
		}
	};
	($t:ty, $($others:ty),+) => {
		impl_round_for!($t);
		impl_round_for!($($others),+);
	};
}

impl_round_for!(f32, f64);

pub trait TruncToUsize {
	#[must_use]
	fn trunc_usize(self) -> usize;
}

macro_rules! impl_trunc_for {
	($t:ty) => {
		#[allow(clippy::cast_sign_loss)]
		impl TruncToUsize for $t {
			fn trunc_usize(self) -> usize {
				self as usize
			}
		}
	};
	($t:ty, $($others:ty),+) => {
		impl_trunc_for!($t);
		impl_trunc_for!($($others),+);
	};
}

impl_trunc_for!(f32, f64);

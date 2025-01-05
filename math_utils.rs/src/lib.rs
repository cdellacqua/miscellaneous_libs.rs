pub trait MapRange
where
	Self: Sized,
{
	#[must_use]
	fn map(self, in_interval: (Self, Self), out_interval: (Self, Self)) -> Self;
}

pub trait MapRangeClamped
where
	Self: Sized,
{
	#[must_use]
	fn map_clamped(self, in_interval: (Self, Self), out_interval: (Self, Self)) -> Self;
}

pub trait MapRatio
where
	Self: Sized,
{
	#[must_use]
	fn map_ratio(self, out_interval: (Self, Self)) -> Self;
}

pub trait MapRatioClamped
where
	Self: Sized,
{
	#[must_use]
	fn map_ratio_clamped(self, out_interval: (Self, Self)) -> Self;
}

macro_rules! impl_map_range_for {
	($t:ty) => {
		impl MapRange for $t {
			fn map(self, in_interval: ($t, $t), out_interval: ($t, $t)) -> $t {
				(self - in_interval.0) / (in_interval.1 - in_interval.0)
					* (out_interval.1 - out_interval.0)
					+ out_interval.0
			}
		}
	};
	($t:ty, $($others:ty),+) => {
		impl_map_range_for!($t);
		impl_map_range_for!($($others),+);
	};
}

macro_rules! impl_map_range_clamped_for {
	($t:ty) => {
		impl MapRangeClamped for $t {
			fn map_clamped(self, in_interval: ($t, $t), out_interval: ($t, $t)) -> $t {
				((self - in_interval.0) / (in_interval.1 - in_interval.0)
					* (out_interval.1 - out_interval.0)
					+ out_interval.0)
					.clamp(
						<$t>::min(out_interval.0, out_interval.1),
						<$t>::max(out_interval.0, out_interval.1),
					)
			}
		}
	};
	($t:ty, $($others:ty),+) => {
		impl_map_range_clamped_for!($t);
		impl_map_range_clamped_for!($($others),+);
	};
}

macro_rules! impl_map_ratio_for {
	($t:ty) => {
		impl MapRatio for $t {
			fn map_ratio(self, out_interval: ($t, $t)) -> $t {
				self
					* (out_interval.1 - out_interval.0)
					+ out_interval.0
			}
		}
	};
	($t:ty, $($others:ty),+) => {
		impl_map_ratio_for!($t);
		impl_map_ratio_for!($($others),+);
	};
}

macro_rules! impl_map_ratio_clamped_for {
	($t:ty) => {
		impl MapRatioClamped for $t {
			fn map_ratio_clamped(self, out_interval: ($t, $t)) -> $t {
				(self
					* (out_interval.1 - out_interval.0)
					+ out_interval.0)
					.clamp(
						<$t>::min(out_interval.0, out_interval.1),
						<$t>::max(out_interval.0, out_interval.1),
					)
			}
		}
	};
	($t:ty, $($others:ty),+) => {
		impl_map_ratio_clamped_for!($t);
		impl_map_ratio_clamped_for!($($others),+);
	};
}

impl_map_range_for!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize, f32, f64);
impl_map_range_clamped_for!(
	u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize, f32, f64
);
impl_map_ratio_for!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize, f32, f64);
impl_map_ratio_clamped_for!(
	u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize, f32, f64
);

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
	use super::*;

	#[test]
	fn test_f32_map() {
		assert_eq!(0.1.map((0.1, 0.2), (0., 10.)), 0.);
		assert_eq!(0.2.map((0.1, 0.2), (0., 10.)), 10.);
		assert_eq!(0.15.map((0.1, 0.2), (0., 10.)), 4.999_999_999_999_999);
		assert_eq!(0.199.map((0.1, 0.2), (-10., 10.)), 9.8);
		assert_eq!(0.0.map((-0.1, 0.2), (-10., 10.)), -3.333_333_333_333_334);
	}
	#[test]
	fn test_f32_map_ratio() {
		assert_eq!((-0.1).map_ratio((0., 10.)), -1.);
		assert_eq!(0.1.map_ratio((0., 10.)), 1.);
		assert_eq!(0.5.map_ratio((0., 10.)), 5.);
		assert_eq!(1.1.map_ratio((0., 10.)), 11.);
	}
	#[test]
	fn test_f32_map_ratio_clamped() {
		assert_eq!((-0.1).map_ratio_clamped((0., 10.)), 0.);
		assert_eq!(0.1.map_ratio((0., 10.)), 1.);
		assert_eq!(0.5.map_ratio_clamped((0., 10.)), 5.);
		assert_eq!(1.1.map_ratio_clamped((0., 10.)), 10.);
	}
	#[test]
	fn test_f32_map_inverted_out() {
		assert_eq!(0.1.map((0.1, 0.2), (10., 0.)), 10.);
		assert_eq!(0.2.map((0.1, 0.2), (10., 0.)), 0.);
		assert_eq!(0.15.map((0.1, 0.2), (10., 0.)), 5.000_000_000_000_001);
	}
	#[test]
	fn test_f32_map_inverted_in() {
		assert_eq!(0.1.map((0.2, 0.1), (0., 10.)), 10.);
		assert_eq!(0.2.map((0.2, 0.1), (0., 10.)), 0.);
		assert_eq!(0.15.map((0.2, 0.1), (0., 10.)), 5.000_000_000_000_001);
	}
	#[test]
	fn test_f32_map_inverted_in_out() {
		assert_eq!(0.1.map((0.2, 0.1), (10., 0.)), 0.);
		assert_eq!(0.2.map((0.2, 0.1), (10., 0.)), 10.);
		assert_eq!(0.15.map((0.2, 0.1), (10., 0.)), 4.999_999_999_999_999);
	}
}

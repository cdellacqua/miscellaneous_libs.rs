use std::{
	fmt::Display,
	ops::{Add, Div, Mul, Sub},
};

use crate::ext::{DivisibleByUsize, MultiplyByUsize, TruncToUsize};

/// A discrete interval with utility functions to map to and from "bins"
/// from a "continuous" range defined with floating point values (e.g. f32 or f64)
pub struct DiscreteInterval<T> {
	interval: (T, T),
	n_of_bins: usize,
}

impl<
		T: Copy
			+ TruncToUsize
			+ Add<T, Output = T>
			+ Sub<T, Output = T>
			+ Div<T, Output = T>
			+ Mul<T, Output = T>
			+ DivisibleByUsize
			+ MultiplyByUsize
			+ PartialOrd
			+ Display,
	> DiscreteInterval<T>
{
	#[must_use]
	pub fn new(interval: (T, T), n_of_bins: usize) -> Self {
		Self {
			interval,
			n_of_bins,
		}
	}

	#[must_use]
	#[allow(clippy::cast_precision_loss)]
	pub fn value_to_bin_idx(&self, value: T) -> usize {
		debug_assert!(
			value >= self.interval.0 && value <= self.interval.1,
			"value {} is out of range {}..={}",
			value,
			self.interval.0,
			self.interval.1
		);
		((value - self.interval.0) / self.bin_width())
			.trunc_usize()
			.min(self.n_of_bins - 1)
	}

	#[must_use]
	#[allow(clippy::cast_precision_loss)]
	pub fn bin_width(&self) -> T {
		(self.interval.1 - self.interval.0).div_usize(self.n_of_bins)
	}

	#[must_use]
	#[allow(clippy::cast_precision_loss)]
	pub fn bin_idx_to_range_start(&self, bin_idx: usize) -> T {
		debug_assert!(
			bin_idx < self.n_of_bins,
			"index {} is out of range. n_of_bins is {}",
			bin_idx,
			self.n_of_bins
		);
		self.interval.0 + self.bin_width().mul_usize(bin_idx)
	}

	#[must_use]
	#[allow(clippy::cast_precision_loss)]
	pub fn bin_idx_to_range_end(&self, bin_idx: usize) -> T {
		debug_assert!(
			bin_idx < self.n_of_bins,
			"index {} is out of range. n_of_bins is {}",
			bin_idx,
			self.n_of_bins
		);
		self.interval.0 + self.bin_width().mul_usize(bin_idx + 1)
	}

	#[must_use]
	pub fn bin_range(&self, bin_idx: usize) -> (T, T) {
		let gap = self.bin_width();
		let value = self.bin_idx_to_range_start(bin_idx);
		(value, value + gap)
	}

	#[must_use]
	pub fn bin_midpoint(&self, bin_idx: usize) -> T {
		let gap = self.bin_width();
		let half_gap = gap.div_usize(2);
		let value = self.bin_idx_to_range_start(bin_idx);
		value + half_gap
	}
}

#[cfg(test)]
mod tests {
	use super::DiscreteInterval;

	#[test]
	fn test_discrete_with_offset() {
		let interval = DiscreteInterval::new((0f32, 100f32), 10);
		assert_eq!(interval.n_of_bins, 10);
		assert!((interval.bin_width() - 10.).abs() < f32::EPSILON);
		assert!(
			(interval.bin_idx_to_range_start(interval.value_to_bin_idx(100.)) - 90.).abs()
				< f32::EPSILON
		);
		assert!(
			(interval.bin_idx_to_range_end(interval.value_to_bin_idx(100.)) - 100.).abs()
				< f32::EPSILON
		);
	}

	#[test]
	fn test_value_to_bin() {
		let interval = DiscreteInterval::new((0f32, 100f32), 10);
		assert_eq!(interval.n_of_bins, 10);
		assert!((interval.bin_width() - 10.).abs() < f32::EPSILON);
		assert_eq!(interval.value_to_bin_idx(0.), 0);
		assert_eq!(interval.value_to_bin_idx(4.), 0);
		assert_eq!(interval.value_to_bin_idx(6.), 0);
		assert_eq!(interval.value_to_bin_idx(9.), 0);
		assert_eq!(interval.value_to_bin_idx(14.), 1);
		assert_eq!(interval.value_to_bin_idx(16.), 1);
		assert_eq!(interval.value_to_bin_idx(85.), 8);
		assert_eq!(interval.value_to_bin_idx(88.), 8);
		assert_eq!(interval.value_to_bin_idx(96.), 9);
		assert_eq!(interval.value_to_bin_idx(99.), 9);
		assert_eq!(interval.value_to_bin_idx(100.), 9);
	}

	#[test]
	fn test_value_to_bin_with_offset() {
		let interval = DiscreteInterval::new((10f32, 110f32), 10);
		assert_eq!(interval.n_of_bins, 10);
		assert!((interval.bin_width() - 10.).abs() < f32::EPSILON);
		assert_eq!(interval.value_to_bin_idx(10. + 0.), 0);
		assert_eq!(interval.value_to_bin_idx(10. + 4.), 0);
		assert_eq!(interval.value_to_bin_idx(10. + 6.), 0);
		assert_eq!(interval.value_to_bin_idx(10. + 9.), 0);
		assert_eq!(interval.value_to_bin_idx(10. + 14.), 1);
		assert_eq!(interval.value_to_bin_idx(10. + 16.), 1);
		assert_eq!(interval.value_to_bin_idx(10. + 85.), 8);
		assert_eq!(interval.value_to_bin_idx(10. + 88.), 8);
		assert_eq!(interval.value_to_bin_idx(10. + 96.), 9);
		assert_eq!(interval.value_to_bin_idx(10. + 99.), 9);
		assert_eq!(interval.value_to_bin_idx(10. + 100.), 9);
	}

	#[test]
	fn test_value_to_bin_with_negative_offset() {
		let interval = DiscreteInterval::new((-10f32, 90f32), 10);
		assert_eq!(interval.n_of_bins, 10);
		assert!((interval.bin_width() - 10.).abs() < f32::EPSILON);
		assert_eq!(interval.value_to_bin_idx(-10. + 0.), 0);
		assert_eq!(interval.value_to_bin_idx(-10. + 4.), 0);
		assert_eq!(interval.value_to_bin_idx(-10. + 6.), 0);
		assert_eq!(interval.value_to_bin_idx(-10. + 9.), 0);
		assert_eq!(interval.value_to_bin_idx(-10. + 14.), 1);
		assert_eq!(interval.value_to_bin_idx(-10. + 16.), 1);
		assert_eq!(interval.value_to_bin_idx(-10. + 85.), 8);
		assert_eq!(interval.value_to_bin_idx(-10. + 88.), 8);
		assert_eq!(interval.value_to_bin_idx(-10. + 96.), 9);
		assert_eq!(interval.value_to_bin_idx(-10. + 99.), 9);
		assert_eq!(interval.value_to_bin_idx(-10. + 100.), 9);
	}
}

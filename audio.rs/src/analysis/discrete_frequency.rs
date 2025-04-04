use std::{
	fmt::Debug,
	ops::{Add, AddAssign, Sub, SubAssign},
};

use derive_more::derive::From;
use math_utils::discrete_interval::DiscreteInterval;

/// A [`DiscreteInterval`] instance that describes the DFT bins
/// as a sequence of bins, centered around their respective frequencies.
///
/// Note that bin 0 is centered at 0Hz, which implies that it's range is from `(-bin_width / 2, +bin_width / 2)`.
/// Also note that this discrete interval includes the Nyquist frequency (`bin_idx == samples_per_window / 2`), which is centered around `sample_rate / 2`, therefore
/// its range is `(sample_rate / 2 - bin_width / 2, sample_rate / 2 + bin_width / 2)`.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn dft_frequency_interval(
	sample_rate: usize,
	samples_per_window: usize,
) -> DiscreteInterval<f32> {
	DiscreteInterval::new(
		(
			-(sample_rate as f32 / 2. / samples_per_window as f32),
			sample_rate as f32 / 2. + (sample_rate as f32 / 2. / samples_per_window as f32),
		),
		n_of_frequency_bins(samples_per_window),
	)
}

#[must_use]
pub fn frequency_to_bin_idx(
	sample_rate: usize,
	samples_per_window: usize,
	frequency: f32,
) -> usize {
	dft_frequency_interval(sample_rate, samples_per_window).value_to_bin(frequency)
}

#[must_use]
pub fn frequency_gap(sample_rate: usize, samples_per_window: usize) -> f32 {
	dft_frequency_interval(sample_rate, samples_per_window).bin_width()
}

#[must_use]
pub fn bin_idx_to_frequency(sample_rate: usize, samples_per_window: usize, bin_idx: usize) -> f32 {
	dft_frequency_interval(sample_rate, samples_per_window).bin_midpoint(bin_idx)
}

#[must_use]
pub fn frequency_interval(
	sample_rate: usize,
	samples_per_window: usize,
	bin_idx: usize,
) -> (f32, f32) {
	dft_frequency_interval(sample_rate, samples_per_window).bin_range(bin_idx)
}

#[must_use]
pub fn all_frequency_bins(sample_rate: usize, samples_per_window: usize) -> Vec<DiscreteFrequency> {
	(0..n_of_frequency_bins(samples_per_window))
		.map(|bin_idx| DiscreteFrequency::new(sample_rate, samples_per_window, bin_idx))
		.collect()
}

/// DFT results are mirrored.
///
/// When `samples_per_window == sample_rate`, the range includes all the indices that correspond to
/// the frequencies between 0 and the Nyquist frequency.
#[must_use]
pub const fn n_of_frequency_bins(samples_per_window: usize) -> usize {
	samples_per_window / 2 + 1
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, From, Default)]
pub struct DiscreteFrequency {
	sample_rate: usize,
	samples_per_window: usize,
	bin_idx: usize,
}

impl Debug for DiscreteFrequency {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("DiscreteFrequency")
			.field("sample_rate", &self.sample_rate)
			.field("samples_per_window", &self.samples_per_window)
			.field("bin_idx", &self.bin_idx)
			.field("frequency()", &self.frequency())
			.field("frequency_interval()", &self.frequency_interval())
			.finish()
	}
}

impl DiscreteFrequency {
	#[must_use]
	pub const fn new(sample_rate: usize, samples_per_window: usize, bin_idx: usize) -> Self {
		Self {
			sample_rate,
			samples_per_window,
			bin_idx,
		}
	}

	#[must_use]
	pub fn frequency(&self) -> f32 {
		bin_idx_to_frequency(self.sample_rate, self.samples_per_window, self.bin_idx)
	}

	#[must_use]
	pub fn frequency_interval(&self) -> (f32, f32) {
		frequency_interval(self.sample_rate, self.samples_per_window, self.bin_idx)
	}

	#[must_use]
	pub const fn bin_idx(&self) -> usize {
		self.bin_idx
	}

	#[must_use]
	pub fn from_frequency(sample_rate: usize, samples_per_window: usize, frequency: f32) -> Self {
		Self::new(
			sample_rate,
			samples_per_window,
			frequency_to_bin_idx(sample_rate, samples_per_window, frequency),
		)
	}

	#[must_use]
	pub fn sample_rate(&self) -> usize {
		self.sample_rate
	}

	#[must_use]
	pub fn samples_per_window(&self) -> usize {
		self.samples_per_window
	}
}

impl Add<usize> for DiscreteFrequency {
	type Output = Self;

	fn add(self, rhs: usize) -> Self::Output {
		Self::new(
			self.sample_rate,
			self.samples_per_window,
			self.bin_idx + rhs,
		)
	}
}

impl AddAssign<usize> for DiscreteFrequency {
	fn add_assign(&mut self, rhs: usize) {
		self.bin_idx += rhs;
	}
}

impl Sub<usize> for DiscreteFrequency {
	type Output = Self;

	fn sub(self, rhs: usize) -> Self::Output {
		Self::new(
			self.sample_rate,
			self.samples_per_window,
			self.bin_idx - rhs,
		)
	}
}

impl SubAssign<usize> for DiscreteFrequency {
	fn sub_assign(&mut self, rhs: usize) {
		self.bin_idx -= rhs;
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	#[allow(clippy::cast_precision_loss)]
	fn frequency_intervals() {
		const SAMPLE_RATE: usize = 44100;
		const SAMPLES_PER_WINDOW: usize = 100;

		assert!(
			(DiscreteFrequency::new(SAMPLE_RATE, SAMPLES_PER_WINDOW, 0)
				.frequency_interval()
				.0 + 220.5)
				.abs() < f32::EPSILON,
			"{:?}",
			DiscreteFrequency::new(SAMPLE_RATE, SAMPLES_PER_WINDOW, 0).frequency_interval()
		);
		assert!(
			(DiscreteFrequency::new(SAMPLE_RATE, SAMPLES_PER_WINDOW, 0)
				.frequency_interval()
				.1 - 220.5)
				.abs() < f32::EPSILON,
			"{:?}",
			DiscreteFrequency::new(SAMPLE_RATE, SAMPLES_PER_WINDOW, 0).frequency_interval()
		);
	}

	#[test]
	#[allow(clippy::cast_precision_loss)]
	fn frequency_to_bin_idx_and_viceversa() {
		const SAMPLE_RATE: usize = 44100;
		const SAMPLES_PER_WINDOW: usize = 44100;

		for samples_per_window in (1..=SAMPLES_PER_WINDOW).step_by(21) {
			for i in 0..=samples_per_window / 2 {
				assert_eq!(
					i,
					DiscreteFrequency::from_frequency(
						SAMPLE_RATE,
						SAMPLES_PER_WINDOW,
						DiscreteFrequency::new(SAMPLE_RATE, SAMPLES_PER_WINDOW, i).frequency()
					)
					.bin_idx(),
					"{samples_per_window:?}"
				);
				assert!(
					DiscreteFrequency::new(SAMPLE_RATE, SAMPLES_PER_WINDOW, i).bin_idx()
						< samples_per_window
				);
			}
		}
	}

	#[test]
	#[allow(clippy::cast_precision_loss)]
	fn nyquist() {
		const SAMPLE_RATE: usize = 44100;
		const SAMPLES_PER_WINDOW: usize = 44100;

		assert!(
			DiscreteFrequency::new(
				SAMPLE_RATE,
				SAMPLES_PER_WINDOW,
				n_of_frequency_bins(SAMPLES_PER_WINDOW) - 1
			)
			.bin_idx() == SAMPLES_PER_WINDOW / 2
		);
		assert!(
			(DiscreteFrequency::new(
				SAMPLE_RATE,
				SAMPLES_PER_WINDOW,
				n_of_frequency_bins(SAMPLES_PER_WINDOW) - 1
			)
			.frequency() - SAMPLE_RATE as f32 / 2.)
				.abs() < f32::EPSILON,
		);
	}
}

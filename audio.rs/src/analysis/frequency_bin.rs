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
/// Also note that this discrete interval includes the Nyquist frequency (`bin_idx == samples / 2`), which is centered around `sample_rate / 2`, therefore
/// its range is `(sample_rate / 2 - bin_width / 2, sample_rate / 2 + bin_width / 2)`.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn dft_frequency_interval(
	sample_rate: usize,
	samples: usize,
) -> DiscreteInterval<f32> {
	DiscreteInterval::new(
		(
			-(sample_rate as f32 / 2. / samples as f32),
			sample_rate as f32 / 2. + (sample_rate as f32 / 2. / samples as f32),
		),
		n_of_frequency_bins(samples),
	)
}

#[must_use]
pub fn frequency_to_bin_idx(sample_rate: usize, samples: usize, frequency: f32) -> usize {
	dft_frequency_interval(sample_rate, samples).value_to_bin(frequency)
}

#[must_use]
pub fn frequency_gap(sample_rate: usize, samples: usize) -> f32 {
	dft_frequency_interval(sample_rate, samples).bin_width()
}

#[must_use]
pub fn bin_idx_to_frequency(sample_rate: usize, samples: usize, bin_idx: usize) -> f32 {
	dft_frequency_interval(sample_rate, samples).bin_midpoint(bin_idx)
}

#[must_use]
pub fn frequency_interval(sample_rate: usize, samples: usize, bin_idx: usize) -> (f32, f32) {
	dft_frequency_interval(sample_rate, samples).bin_range(bin_idx)
}

#[must_use]
pub fn all_frequency_bins(sample_rate: usize, samples: usize) -> Vec<DynFrequencyBin> {
	(0..n_of_frequency_bins(samples))
		.map(|bin_idx| DynFrequencyBin::new(sample_rate, samples, bin_idx))
		.collect()
}

/// DFT results are mirrored.
///
/// When samples == sample rate, the range includes all the indices that correspond to
/// the frequencies between 0 and the Nyquist frequency.
#[must_use]
pub const fn n_of_frequency_bins(samples: usize) -> usize {
	samples / 2 + 1
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, From, Default)]
pub struct FrequencyBin<const SAMPLE_RATE: usize, const SAMPLES: usize> {
	bin_idx: usize,
}

impl<const SAMPLE_RATE: usize, const SAMPLES: usize> Debug for FrequencyBin<SAMPLE_RATE, SAMPLES> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("FrequencyBin")
			.field("bin_idx", &self.bin_idx)
			.field("frequency_interval()", &self.frequency_interval())
			.finish()
	}
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct DynFrequencyBin {
	sample_rate: usize,
	samples: usize,
	bin_idx: usize,
}

impl Debug for DynFrequencyBin {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("DynFrequencyBin")
			.field("sample_rate", &self.sample_rate)
			.field("samples", &self.samples)
			.field("bin_idx", &self.bin_idx)
			.field("frequency_interval()", &self.frequency_interval())
			.finish()
	}
}

impl DynFrequencyBin {
	#[must_use]
	pub const fn new(sample_rate: usize, samples: usize, bin_idx: usize) -> Self {
		Self {
			sample_rate,
			samples,
			bin_idx,
		}
	}

	#[must_use]
	pub fn from_frequency(sample_rate: usize, samples: usize, frequency: f32) -> Self {
		let bin_idx = frequency_to_bin_idx(sample_rate, samples, frequency);
		Self {
			sample_rate,
			samples,
			bin_idx,
		}
	}

	#[must_use]
	pub fn frequency(&self) -> f32 {
		bin_idx_to_frequency(self.sample_rate, self.samples, self.bin_idx)
	}

	#[must_use]
	pub fn frequency_interval(&self) -> (f32, f32) {
		frequency_interval(self.sample_rate, self.samples, self.bin_idx)
	}

	#[must_use]
	pub fn bin_idx(&self) -> usize {
		self.bin_idx
	}

	#[must_use]
	pub fn sample_rate(&self) -> usize {
		self.sample_rate
	}

	#[must_use]
	pub fn samples(&self) -> usize {
		self.samples
	}
}

impl<const SAMPLE_RATE: usize, const SAMPLES: usize> FrequencyBin<SAMPLE_RATE, SAMPLES> {
	#[must_use]
	pub const fn new(bin_idx: usize) -> Self {
		Self { bin_idx }
	}

	#[must_use]
	pub fn frequency(&self) -> f32 {
		bin_idx_to_frequency(SAMPLE_RATE, SAMPLES, self.bin_idx)
	}

	#[must_use]
	pub fn frequency_interval(&self) -> (f32, f32) {
		frequency_interval(SAMPLE_RATE, SAMPLES, self.bin_idx)
	}

	#[must_use]
	pub const fn bin_idx(&self) -> usize {
		self.bin_idx
	}

	#[must_use]
	pub fn from_frequency(frequency: f32) -> Self {
		Self::new(frequency_to_bin_idx(SAMPLE_RATE, SAMPLES, frequency))
	}

	#[must_use]
	pub fn sample_rate(&self) -> usize {
		SAMPLE_RATE
	}

	#[must_use]
	pub fn samples(&self) -> usize {
		SAMPLES
	}
}

impl<const SAMPLE_RATE: usize, const SAMPLES: usize> Add<usize>
	for FrequencyBin<SAMPLE_RATE, SAMPLES>
{
	type Output = Self;

	fn add(self, rhs: usize) -> Self::Output {
		Self::new(self.bin_idx + rhs)
	}
}

impl<const SAMPLE_RATE: usize, const SAMPLES: usize> AddAssign<usize>
	for FrequencyBin<SAMPLE_RATE, SAMPLES>
{
	fn add_assign(&mut self, rhs: usize) {
		self.bin_idx += rhs;
	}
}

impl<const SAMPLE_RATE: usize, const SAMPLES: usize> Sub<usize>
	for FrequencyBin<SAMPLE_RATE, SAMPLES>
{
	type Output = Self;

	fn sub(self, rhs: usize) -> Self::Output {
		Self::new(self.bin_idx - rhs)
	}
}

impl<const SAMPLE_RATE: usize, const SAMPLES: usize> SubAssign<usize>
	for FrequencyBin<SAMPLE_RATE, SAMPLES>
{
	fn sub_assign(&mut self, rhs: usize) {
		self.bin_idx -= rhs;
	}
}

impl Add<usize> for DynFrequencyBin {
	type Output = Self;

	fn add(self, rhs: usize) -> Self::Output {
		Self::new(self.sample_rate, self.samples, self.bin_idx + rhs)
	}
}

impl AddAssign<usize> for DynFrequencyBin {
	fn add_assign(&mut self, rhs: usize) {
		self.bin_idx += rhs;
	}
}

impl Sub<usize> for DynFrequencyBin {
	type Output = Self;

	fn sub(self, rhs: usize) -> Self::Output {
		Self::new(self.sample_rate, self.samples, self.bin_idx - rhs)
	}
}

impl SubAssign<usize> for DynFrequencyBin {
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
		const SAMPLES: usize = 100;

		assert!(
			(FrequencyBin::<SAMPLE_RATE, SAMPLES>::new(0)
				.frequency_interval()
				.0 + 220.5)
				.abs() < f32::EPSILON,
			"{:?}",
			FrequencyBin::<SAMPLE_RATE, SAMPLES>::new(0).frequency_interval()
		);
		assert!(
			(FrequencyBin::<SAMPLE_RATE, SAMPLES>::new(0)
				.frequency_interval()
				.1 - 220.5)
				.abs() < f32::EPSILON,
			"{:?}",
			FrequencyBin::<SAMPLE_RATE, SAMPLES>::new(0).frequency_interval()
		);
	}

	#[test]
	#[allow(clippy::cast_precision_loss)]
	fn frequency_to_bin_idx_and_viceversa() {
		const SAMPLE_RATE: usize = 44100;
		const SAMPLES: usize = 44100;

		for samples in (1..=SAMPLES).step_by(21) {
			for i in 0..=samples / 2 {
				assert_eq!(
					i,
					FrequencyBin::<SAMPLE_RATE, SAMPLES>::from_frequency(
						FrequencyBin::<SAMPLE_RATE, SAMPLES>::new(i).frequency()
					)
					.bin_idx(),
					"{samples:?}"
				);
				assert!(FrequencyBin::<SAMPLE_RATE, SAMPLES>::new(i).bin_idx() < samples);
			}
		}
	}

	#[test]
	#[allow(clippy::cast_precision_loss)]
	fn nyquist() {
		const SAMPLE_RATE: usize = 44100;
		const SAMPLES: usize = 44100;

		assert!(
			FrequencyBin::<SAMPLE_RATE, SAMPLES>::new(n_of_frequency_bins(SAMPLES) - 1).bin_idx()
				== SAMPLES / 2
		);
		assert!(
			(FrequencyBin::<SAMPLE_RATE, SAMPLES>::new(n_of_frequency_bins(SAMPLES) - 1)
				.frequency() - SAMPLE_RATE as f32 / 2.)
				.abs() < f32::EPSILON,
		);
	}
}

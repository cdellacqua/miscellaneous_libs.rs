use crate::NOfSamples;

use super::{frequency_to_index, index_to_frequency};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FftPoint<const SAMPLE_RATE: usize, const SAMPLES: usize> {
	pub magnitude: f32,
	pub frequency: f32,
}

impl<const SAMPLE_RATE: usize, const SAMPLES: usize> FftPoint<SAMPLE_RATE, SAMPLES> {
	#[must_use]
	pub const fn frequency_idx(&self) -> usize {
		frequency_to_index(self.frequency, NOfSamples::<SAMPLE_RATE>::new(SAMPLES))
	}

	#[must_use]
	pub const fn to_fft_bin_point(&self) -> FftBinPoint<SAMPLE_RATE, SAMPLES> {
		FftBinPoint {
			magnitude: self.magnitude,
			frequency_idx: self.frequency_idx(),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FftBinPoint<const SAMPLE_RATE: usize, const SAMPLES: usize> {
	pub magnitude: f32,
	pub frequency_idx: usize,
}

impl<const SAMPLE_RATE: usize, const SAMPLES: usize> FftBinPoint<SAMPLE_RATE, SAMPLES> {
	#[must_use]
	pub const fn frequency(&self) -> f32 {
		index_to_frequency(self.frequency_idx, NOfSamples::<SAMPLE_RATE>::new(SAMPLES))
	}

	#[must_use]
	pub const fn to_fft_point(&self) -> FftPoint<SAMPLE_RATE, SAMPLES> {
		FftPoint {
			magnitude: self.magnitude,
			frequency: self.frequency(),
		}
	}
}

use super::{frequency_to_index, index_to_frequency};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FftPoint {
	pub magnitude: f32,
	pub frequency: f32,
}

impl FftPoint {
	#[must_use]
	pub fn frequency_idx(&self, sample_rate: usize, samples: usize) -> usize {
		frequency_to_index(self.frequency, sample_rate, samples)
	}

	#[must_use]
	pub fn to_fft_bin_point(&self, sample_rate: usize, samples: usize) -> FftBinPoint {
		FftBinPoint {
			magnitude: self.magnitude,
			frequency_idx: self.frequency_idx(sample_rate, samples),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FftBinPoint {
	pub magnitude: f32,
	pub frequency_idx: usize,
}

impl FftBinPoint {
	#[must_use]
	pub fn frequency(&self, sample_rate: usize, samples: usize) -> f32 {
		index_to_frequency(self.frequency_idx, sample_rate, samples)
	}

	#[must_use]
	pub fn to_fft_point(&self, sample_rate: usize, samples: usize) -> FftPoint {
		FftPoint {
			magnitude: self.magnitude,
			frequency: self.frequency(sample_rate, samples),
		}
	}
}

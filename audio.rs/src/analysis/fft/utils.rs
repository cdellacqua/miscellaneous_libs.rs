use std::ops::RangeInclusive;

use math_utils::const_num::round_f32_to_usize;

use crate::NOfSamples;

/// FFT results are mirrored.
///
/// When samples == sample rate, the range includes all the indices that correspond to
/// the frequencies between 0 and the Nyquist frequency.
#[must_use]
pub const fn fft_real_length(samples: usize) -> usize {
	samples / 2 + 1
}

#[must_use]
pub const fn index_to_frequency<const SAMPLE_RATE: usize>(
	i: usize,
	samples: NOfSamples<SAMPLE_RATE>,
) -> f32 {
	#[allow(clippy::cast_precision_loss)]
	return i as f32 * samples.sample_rate() as f32 / samples.num() as f32;
}

#[must_use]
pub const fn frequency_to_index<const SAMPLE_RATE: usize>(
	frequency: f32,
	samples: NOfSamples<SAMPLE_RATE>,
) -> usize {
	#[allow(clippy::cast_precision_loss)]
	#[allow(clippy::cast_sign_loss)]
	return round_f32_to_usize(frequency / samples.sample_rate() as f32 * samples.num() as f32);
}

pub fn fft_frequency_bins<const SAMPLE_RATE: usize>(
	samples: NOfSamples<SAMPLE_RATE>,
) -> impl Iterator<Item = f32> {
	(0..fft_real_length(samples.num())).map(move |i| index_to_frequency(i, samples))
}

/// # Panics
/// - if the filter results in an empty set of frequencies.
#[must_use]
pub const fn filtered_frequency_index_range<const SAMPLE_RATE: usize>(
	samples: NOfSamples<SAMPLE_RATE>,
	frequency_range: (f32, f32),
) -> RangeInclusive<usize> {
	let start = {
		let len = fft_real_length(samples.num());
		let mut idx = 0;
		loop {
			assert!(
				idx < len,
				"expected at least one valid frequency in the specified range"
			);

			if index_to_frequency(idx, samples) > frequency_range.0 {
				if idx == 0 {
					break 0;
				}
				break idx - 1;
			}

			#[allow(clippy::float_cmp)]
			if index_to_frequency(idx, samples) == frequency_range.0 {
				break idx;
			}

			idx += 1;
		}
	};

	let end = {
		let len = fft_real_length(samples.num());
		let mut i = 0;
		loop {
			assert!(
				i < len,
				"expected at least one valid frequency in the specified range"
			);
			let idx = len - 1 - i;

			if index_to_frequency(idx, samples) < frequency_range.1 {
				if idx == len - 1 {
					break len - 1;
				}
				break idx + 1;
			}

			#[allow(clippy::float_cmp)]
			if index_to_frequency(idx, samples) == frequency_range.1 {
				break idx;
			}

			i += 1;
		}
	};

	start..=end
}

#[cfg(test)]
mod tests {
	use crate::NOfSamples;

use super::{filtered_frequency_index_range, frequency_to_index, index_to_frequency};

	#[test]
	fn frequency_to_index_and_viceversa() {
		const SAMPLE_RATE: usize = 44100;

		for samples in 1..=54100 {
			if samples % 100 == 0 {
				println!("{samples}");
			}
			for i in 0..samples {
				assert_eq!(
					i,
					frequency_to_index(
						index_to_frequency(i, NOfSamples::<SAMPLE_RATE>::new(samples)),
						NOfSamples::<SAMPLE_RATE>::new(samples)
					)
				);
				assert!(
					frequency_to_index(
						index_to_frequency(i, NOfSamples::<SAMPLE_RATE>::new(samples)),
						NOfSamples::<SAMPLE_RATE>::new(samples)
					) < samples
				);
			}
		}
	}

	#[test]
	fn test_filtered_frequency_index_range() {
		assert_eq!(
			filtered_frequency_index_range(NOfSamples::<44100>::new(44100), (19_000., 20_000.)),
			(19_000..=20_000)
		);
		assert_eq!(
			filtered_frequency_index_range(NOfSamples::<44100>::new(44100), (19_001.5, 20_001.5)),
			(19_001..=20_002)
		);
	}
}

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
	return i as f32 * samples.sample_rate() as f32 / samples.inner() as f32;
}

#[must_use]
pub const fn frequency_to_index<const SAMPLE_RATE: usize>(
	frequency: f32,
	samples: NOfSamples<SAMPLE_RATE>,
) -> usize {
	#[allow(clippy::cast_precision_loss)]
	#[allow(clippy::cast_sign_loss)]
	return round_f32_to_usize(frequency / samples.sample_rate() as f32 * samples.inner() as f32);
}

pub fn fft_frequency_bins<const SAMPLE_RATE: usize>(
	samples: NOfSamples<SAMPLE_RATE>,
) -> impl Iterator<Item = f32> {
	(0..fft_real_length(*samples)).map(move |i| index_to_frequency(i, samples))
}

#[cfg(test)]
mod tests {
	use crate::NOfSamples;

	use super::{frequency_to_index, index_to_frequency};

	#[test]
	fn frequency_to_index_and_viceversa() {
		const SAMPLE_RATE: usize = 44100;

		for samples in 1..=54100 {
			if samples % 1000 == 0 {
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
}

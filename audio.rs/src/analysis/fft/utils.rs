use std::ops::RangeInclusive;

/// FFT results are mirrored.
///
/// When samples == sample rate, the range includes all the indices that correspond to
/// the frequencies between 0 and the Nyquist frequency.
#[must_use]
pub fn fft_real_length(samples: usize) -> usize {
	samples / 2 + 1
}

#[must_use]
pub fn index_to_frequency(i: usize, sample_rate: usize, samples: usize) -> f32 {
	#[allow(clippy::cast_precision_loss)]
	return i as f32 * sample_rate as f32 / samples as f32;
}

#[must_use]
pub fn frequency_to_index(frequency: f32, sample_rate: usize, samples: usize) -> usize {
	#[allow(clippy::cast_precision_loss)]
	#[allow(clippy::cast_sign_loss)]
	return (frequency / sample_rate as f32 * samples as f32).round() as usize;
}

pub fn fft_frequency_bins(sample_rate: usize, samples: usize) -> impl Iterator<Item = f32> {
	(0..fft_real_length(samples)).map(move |i| index_to_frequency(i, sample_rate, samples))
}

/// # Panics
/// - if the filter results in an empty set of frequencies.
#[must_use]
pub fn filtered_frequency_index_range(
	sample_rate: usize,
	samples: usize,
	frequency_range: (f32, f32),
) -> RangeInclusive<usize> {
	let start = (0..fft_real_length(samples))
		.find(|&i| index_to_frequency(i, sample_rate, samples) >= frequency_range.0)
		.expect("at least one valid frequency in the specified range");
	let end = (0..fft_real_length(samples))
		.rev()
		.find(|&i| index_to_frequency(i, sample_rate, samples) <= frequency_range.1)
		.expect("at least one valid frequency in the specified range");

	start..=end
}

#[cfg(test)]
mod tests {
	use super::{frequency_to_index, index_to_frequency};

	#[test]
	fn frequency_to_index_and_viceversa() {
		let sample_rate = 44100;

		for samples in 1..=54100 {
			if samples % 100 == 0 {
				println!("{samples}");
			}
			for i in 0..samples {
				assert_eq!(
					i,
					frequency_to_index(
						index_to_frequency(i, sample_rate, samples),
						sample_rate,
						samples
					)
				);
				assert!(
					frequency_to_index(
						index_to_frequency(i, sample_rate, samples),
						sample_rate,
						samples
					) < samples
				);
			}
		}
	}
}

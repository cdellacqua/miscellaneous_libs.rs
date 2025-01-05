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

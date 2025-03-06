use std::sync::Arc;

use rustfft::{
	num_complex::{Complex, Complex32},
	Fft, FftPlanner,
};

use crate::analysis::{
	n_of_frequency_bins, windowing_fns::HannWindow, FrequencyBin, Harmonic, WindowingFn,
};

#[derive(Clone)]
pub struct StftAnalyzer<const SAMPLE_RATE: usize, const SAMPLES_PER_WINDOW: usize> {
	windowing_values: Vec<f32>,
	fft_processor: Arc<dyn Fft<f32>>,
	complex_signal: Vec<Complex32>,
	cur_transform_bins: Vec<Harmonic<SAMPLE_RATE, SAMPLES_PER_WINDOW>>,
	normalization_factor: f32,
}

impl<const SAMPLE_RATE: usize, const SAMPLES_PER_WINDOW: usize> std::fmt::Debug
	for StftAnalyzer<SAMPLE_RATE, SAMPLES_PER_WINDOW>
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct(&format!(
			"StftAnalyzer<{SAMPLE_RATE}, {SAMPLES_PER_WINDOW}>"
		))
		.field("windowing_values", &self.windowing_values)
		.field("fft_processor", &"omitted")
		.field("complex_signal", &self.complex_signal)
		.field("cur_transform_bins", &self.cur_transform_bins)
		.field("normalization_factor", &self.normalization_factor)
		.finish()
	}
}

impl<const SAMPLE_RATE: usize, const SAMPLES_PER_WINDOW: usize>
	StftAnalyzer<SAMPLE_RATE, SAMPLES_PER_WINDOW>
{
	#[must_use]
	pub fn new(windowing_fn: &impl WindowingFn) -> Self {
		let mut planner = FftPlanner::new();
		let transform_size = n_of_frequency_bins(SAMPLES_PER_WINDOW);
		Self {
			windowing_values: (0..SAMPLES_PER_WINDOW)
				.map(|i| windowing_fn.ratio_at(i, SAMPLES_PER_WINDOW))
				.collect(),
			fft_processor: planner.plan_fft_forward(SAMPLES_PER_WINDOW),
			complex_signal: vec![Complex { re: 0., im: 0. }; SAMPLES_PER_WINDOW],
			cur_transform_bins: vec![Harmonic::default(); transform_size],
			// https://docs.rs/rustfft/6.2.0/rustfft/index.html#normalization
			#[allow(clippy::cast_precision_loss)]
			normalization_factor: 1.0 / (SAMPLES_PER_WINDOW as f32).sqrt(),
		}
	}

	/// Analyze a signal in the domain of time, sampled at the configured sample rate.
	///
	/// The returned `Vec` is sorted by frequency bin.
	///
	/// Note: performance-wise, FFT works better when the signal length is a power of two.
	///
	/// # Panics
	/// - if the passed `signal` is not compatible with the configured `samples_per_window`.
	#[must_use]
	pub fn analyze(&mut self, signal: &[f32]) -> &Vec<Harmonic<SAMPLE_RATE, SAMPLES_PER_WINDOW>> {
		let samples = signal.len();

		assert_eq!(
			samples, SAMPLES_PER_WINDOW,
			"signal with incompatible length received"
		);

		for ((c, sample), windowing_value) in self
			.complex_signal
			.iter_mut()
			.zip(signal)
			.zip(self.windowing_values.iter())
		{
			*c = Complex::new(sample * windowing_value, 0.0);
		}

		self.fft_processor.process(&mut self.complex_signal);

		let transform_size = self.cur_transform_bins.len();
		self.cur_transform_bins
			.iter_mut()
			.zip(self.complex_signal.iter().take(transform_size))
			.enumerate()
			.for_each(|(i, (dst, src))| {
				*dst = Harmonic::new(src * self.normalization_factor, FrequencyBin::new(i));
			});

		&self.cur_transform_bins
	}

	#[must_use]
	pub fn sample_rate(&self) -> usize {
		SAMPLE_RATE
	}

	#[must_use]
	pub fn samples_per_window(&self) -> usize {
		SAMPLES_PER_WINDOW
	}
}

impl<const SAMPLE_RATE: usize, const SAMPLES_PER_WINDOW: usize> Default
	for StftAnalyzer<SAMPLE_RATE, SAMPLES_PER_WINDOW>
{
	fn default() -> Self {
		Self::new(&HannWindow)
	}
}

#[cfg(test)]
#[cfg(feature = "output")]
mod tests {
	use math_utils::one_dimensional_mapping::MapRatio;

	use crate::{analysis::all_frequency_bins, output::frequencies_to_samples};

	use super::*;

	#[test]
	#[allow(clippy::cast_precision_loss)]
	fn stft_peaks_at_frequency_bin() {
		const SAMPLE_RATE: usize = 44100;
		const SAMPLES: usize = 44100;

		let mut stft_analyzer = StftAnalyzer::<SAMPLE_RATE, SAMPLES>::default();
		let bins = all_frequency_bins(SAMPLE_RATE, SAMPLES);
		let delta_hz = bins[1].frequency() - bins[0].frequency();

		for i in 0..100 {
			let frequency = (i as f32 / 100.0).map_ratio((
				bins[10].frequency() - delta_hz / 2.,
				bins[10].frequency() + delta_hz / 2.,
			));

			let signal = frequencies_to_samples::<SAMPLE_RATE>(SAMPLES, &[frequency], 0.);
			let analysis = stft_analyzer.analyze(signal.as_mono());
			assert!(
				(analysis
					.iter()
					.max_by(|a, b| a.power().total_cmp(&b.power()))
					.unwrap()
					.frequency() - bins[10].frequency())
				.abs() < f32::EPSILON,
				"{frequency} {}", bins[10].frequency()
			);
		}
	}

	#[test]
	#[allow(clippy::cast_precision_loss)]
	fn stft_peaks_at_frequency_bin_440() {
		const SAMPLE_RATE: usize = 44100;
		const SAMPLES: usize = 100;

		let mut stft_analyzer = StftAnalyzer::<SAMPLE_RATE, SAMPLES>::default();
		let signal = frequencies_to_samples::<SAMPLE_RATE>(SAMPLES, &[440.], 0.);
		let analysis = stft_analyzer.analyze(signal.as_mono());
		assert_eq!(
			analysis
				.iter()
				.max_by(|a, b| a.power().total_cmp(&b.power()))
				.unwrap()
				.bin_idx(),
			1
		);
	}
}

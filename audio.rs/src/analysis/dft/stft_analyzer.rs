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
	windowing_fn: Arc<dyn WindowingFn + Sync + Send + 'static>,
	fft_processor: Arc<dyn Fft<f32>>,
	complex_signal: Vec<Complex32>,
	cur_transform_bins: Vec<Harmonic<SAMPLE_RATE, SAMPLES_PER_WINDOW>>,
}

impl<const SAMPLE_RATE: usize, const SAMPLES_PER_WINDOW: usize> std::fmt::Debug
	for StftAnalyzer<SAMPLE_RATE, SAMPLES_PER_WINDOW>
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct(&format!(
			"StftAnalyzer<{SAMPLE_RATE}, {SAMPLES_PER_WINDOW}>"
		))
		.field("windowing_fn", &"omitted")
		.field("fft_processor", &"omitted")
		.field("complex_signal", &self.complex_signal)
		.field("cur_transform_bins", &self.cur_transform_bins)
		.finish()
	}
}

impl<const SAMPLE_RATE: usize, const SAMPLES_PER_WINDOW: usize>
	StftAnalyzer<SAMPLE_RATE, SAMPLES_PER_WINDOW>
{
	#[must_use]
	pub fn new(windowing_fn: impl WindowingFn + Send + Sync + 'static) -> Self {
		let mut planner = FftPlanner::new();
		let transform_size = n_of_frequency_bins(SAMPLES_PER_WINDOW);
		Self {
			windowing_fn: Arc::new(windowing_fn),
			fft_processor: planner.plan_fft_forward(SAMPLES_PER_WINDOW),
			complex_signal: vec![Complex { re: 0., im: 0. }; SAMPLES_PER_WINDOW],
			cur_transform_bins: vec![Harmonic::default(); transform_size],
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
	pub fn analyze_bins(
		&mut self,
		signal: &[f32],
	) -> &Vec<Harmonic<SAMPLE_RATE, SAMPLES_PER_WINDOW>> {
		let samples = signal.len();

		assert_eq!(
			samples, SAMPLES_PER_WINDOW,
			"signal with incompatible length received"
		);

		for (i, (c, sample)) in self.complex_signal.iter_mut().zip(signal).enumerate() {
			*c = Complex::new(
				sample * (self.windowing_fn).ratio_at(i, SAMPLES_PER_WINDOW),
				0.0,
			);
		}

		self.fft_processor.process(&mut self.complex_signal);

		// https://docs.rs/rustfft/6.2.0/rustfft/index.html#normalization
		#[allow(clippy::cast_precision_loss)]
		let normalization_factor = 1.0 / (samples as f32).sqrt();

		let transform_size = self.cur_transform_bins.len();
		self.cur_transform_bins
			.iter_mut()
			.zip(self.complex_signal.iter().take(transform_size))
			.enumerate()
			.for_each(|(i, (dst, src))| {
				*dst = Harmonic::new(src * normalization_factor, FrequencyBin::new(i));
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
		Self::new(HannWindow)
	}
}

#[cfg(test)]
#[cfg(feature = "output")]
mod tests {
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

		// +/-49%
		for i in -49..=49 {
			let delta = delta_hz * (i as f32) / 100.0;

			let signal =
				frequencies_to_samples::<SAMPLE_RATE>(SAMPLES, &[bins[10].frequency() + delta]);
			let analysis = stft_analyzer.analyze_bins(signal.as_mono());
			assert!(
				(analysis
					.iter()
					.max_by(|a, b| a.power().total_cmp(&b.power()))
					.unwrap()
					.frequency() - bins[10].frequency())
				.abs() < f32::EPSILON,
				"{delta}"
			);
		}
	}
}

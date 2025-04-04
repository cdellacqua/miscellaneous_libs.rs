use std::sync::Arc;

use rustfft::{
	num_complex::{Complex, Complex32},
	Fft, FftPlanner,
};

use crate::analysis::{n_of_frequency_bins, DiscreteHarmonic, DiscreteFrequency, WindowingFn};

#[derive(Clone)]
pub struct StftAnalyzer {
	sample_rate: usize,
	samples_per_window: usize,
	windowing_values: Vec<f32>,
	fft_processor: Arc<dyn Fft<f32>>,
	complex_signal: Vec<Complex32>,
	cur_transform_bins: Vec<DiscreteHarmonic>,
	normalization_factor: f32,
}

impl std::fmt::Debug for StftAnalyzer {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("StftAnalyzer")
			.field("sample_rate", &self.sample_rate)
			.field("samples_per_window", &self.samples_per_window)
			.field("windowing_values", &self.windowing_values)
			.field("fft_processor", &"omitted")
			.field("complex_signal", &self.complex_signal)
			.field("cur_transform_bins", &self.cur_transform_bins)
			.field("normalization_factor", &self.normalization_factor)
			.finish()
	}
}

impl StftAnalyzer {
	#[must_use]
	pub fn new(
		sample_rate: usize,
		samples_per_window: usize,
		windowing_fn: &impl WindowingFn,
	) -> Self {
		let mut planner = FftPlanner::new();
		let transform_size = n_of_frequency_bins(samples_per_window);
		Self {
			sample_rate,
			samples_per_window,
			windowing_values: (0..samples_per_window)
				.map(|i| windowing_fn.ratio_at(i, samples_per_window))
				.collect(),
			fft_processor: planner.plan_fft_forward(samples_per_window),
			complex_signal: vec![Complex { re: 0., im: 0. }; samples_per_window],
			cur_transform_bins: (0..transform_size)
				.map(|i| {
					DiscreteHarmonic::new(
						sample_rate,
						samples_per_window,
						Complex::ZERO,
						DiscreteFrequency::new(sample_rate, samples_per_window, i),
					)
				})
				.collect(),
			// https://docs.rs/rustfft/6.2.0/rustfft/index.html#normalization
			#[allow(clippy::cast_precision_loss)]
			normalization_factor: 1.0 / (samples_per_window as f32).sqrt(),
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
	pub fn analyze(&mut self, signal: &[f32]) -> &Vec<DiscreteHarmonic> {
		let samples = signal.len();

		assert_eq!(
			samples, self.samples_per_window,
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
			.for_each(|(dst, src)| {
				dst.phasor = src * self.normalization_factor;
			});

		&self.cur_transform_bins
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

#[cfg(test)]
#[cfg(feature = "output")]
mod tests {
	use math_utils::one_dimensional_mapping::MapRatio;

	use crate::{
		analysis::{all_frequency_bins, windowing_fns::HannWindow, Harmonic},
		output::harmonics_to_samples,
	};

	use super::*;

	#[test]
	#[allow(clippy::cast_precision_loss)]
	fn stft_peaks_at_frequency_bin() {
		const SAMPLE_RATE: usize = 44100;
		const SAMPLES_PER_WINDOW: usize = 44100;

		let mut stft_analyzer = StftAnalyzer::new(SAMPLE_RATE, SAMPLES_PER_WINDOW, &HannWindow);
		let bins = all_frequency_bins(SAMPLE_RATE, SAMPLES_PER_WINDOW);
		let delta_hz = bins[1].frequency() - bins[0].frequency();

		for i in 1..100 {
			let frequency = (i as f32 / 100.0).map_ratio((
				bins[10].frequency() - delta_hz / 2.,
				bins[10].frequency() + delta_hz / 2.,
			));

			let signal = harmonics_to_samples::<SAMPLE_RATE>(
				SAMPLES_PER_WINDOW,
				&[Harmonic::new(Complex32::ONE, frequency)],
			);
			let analysis = stft_analyzer.analyze(signal.as_mono());
			assert!(
				(analysis
					.iter()
					.max_by(|a, b| a.power().total_cmp(&b.power()))
					.unwrap()
					.frequency() - bins[10].frequency())
				.abs() < f32::EPSILON,
				"{frequency} {}",
				bins[10].frequency()
			);
		}
	}

	#[test]
	#[allow(clippy::cast_precision_loss)]
	fn stft_peaks_at_frequency_bin_440() {
		const SAMPLE_RATE: usize = 44100;
		const SAMPLES_PER_WINDOW: usize = 100;

		let mut stft_analyzer = StftAnalyzer::new(SAMPLE_RATE, SAMPLES_PER_WINDOW, &HannWindow);
		let signal = harmonics_to_samples::<SAMPLE_RATE>(
			SAMPLES_PER_WINDOW,
			&[Harmonic::new(Complex32::ONE, 440.)],
		);
		let analysis = stft_analyzer.analyze(signal.as_mono());
		let harmonic = analysis[1..] // skip 0Hz
			.iter()
			.max_by(|a, b| a.power().total_cmp(&b.power()))
			.unwrap();
		assert_eq!(harmonic.bin_idx(), 1);
		assert!(harmonic.phase().abs() < 0.01);
	}

	#[test]
	#[allow(clippy::cast_precision_loss)]
	fn stft_phase() {
		const SAMPLE_RATE: usize = 44100;
		const SAMPLES_PER_WINDOW: usize = 4410;

		let bin = DiscreteFrequency::new(SAMPLE_RATE, SAMPLES_PER_WINDOW, 50);

		let mut stft_analyzer = StftAnalyzer::new(SAMPLE_RATE, SAMPLES_PER_WINDOW, &HannWindow);

		let frequency = bin.frequency();

		let signal = harmonics_to_samples::<SAMPLE_RATE>(
			SAMPLES_PER_WINDOW,
			&[Harmonic::new(Complex32::ONE, frequency)],
		);
		let analysis = stft_analyzer.analyze(signal.as_mono());
		let phase = analysis
			.iter()
			.max_by(|a, b| a.power().total_cmp(&b.power()))
			.unwrap()
			.phase();
		assert!(phase.abs() < 0.001, "{phase}");
	}
}

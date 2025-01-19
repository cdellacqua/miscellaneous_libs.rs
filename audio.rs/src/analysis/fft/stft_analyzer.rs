use std::{ops::RangeInclusive, sync::Arc};

use rustfft::{num_complex::Complex, Fft, FftPlanner};

use crate::analysis::WindowingFn;

use super::{fft_frequency_bins, filtered_frequency_index_range, index_to_frequency, FftPoint};

#[derive(Clone)]
pub struct StftAnalyzer {
	sample_rate: usize,
	samples_per_window: usize,
	windowing_fn: Arc<dyn WindowingFn + Sync + Send + 'static>,
	frequency_indices: RangeInclusive<usize>,
	fft_processor: Arc<dyn Fft<f32>>,
	complex_signal: Vec<Complex<f32>>,
	cur_transform: Vec<FftPoint>,
}

impl std::fmt::Debug for StftAnalyzer {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("StftAnalyzer")
			.field("sample_rate", &self.sample_rate)
			.field("samples_per_window", &self.samples_per_window)
			.field("windowing_fn", &"omitted")
			.field("frequency_indices", &self.frequency_indices)
			.field("fft_processor", &"omitted")
			.field("complex_signal", &self.complex_signal)
			.field("cur_transform", &self.cur_transform)
			.finish()
	}
}

impl StftAnalyzer {
	#[must_use]
	pub fn new(
		sample_rate: usize,
		samples_per_window: usize,
		frequency_range: (f32, f32),
		windowing_fn: impl WindowingFn + Send + Sync + 'static,
	) -> Self {
		let mut planner = FftPlanner::new();
		let frequency_indices =
			filtered_frequency_index_range(sample_rate, samples_per_window, frequency_range);
		Self {
			sample_rate,
			samples_per_window,
			windowing_fn: Arc::new(windowing_fn) as Arc<dyn WindowingFn + Send + Sync + 'static>,

			frequency_indices: frequency_indices.clone(),
			fft_processor: planner.plan_fft_forward(samples_per_window),
			complex_signal: vec![Complex { re: 0., im: 0. }; samples_per_window],
			cur_transform: vec![
				FftPoint {
					magnitude: 0.,
					frequency: 0.
				};
				frequency_indices.count()
			],
		}
	}

	#[must_use]
	pub fn frequency_bins(&self) -> Vec<f32> {
		fft_frequency_bins(self.sample_rate, self.samples_per_window)
			.skip(*self.frequency_indices.start())
			.take(self.frequency_indices.clone().count())
			.collect()
	}

	/// Analyze a signal in the domain of time, sampled at the configured sample rate.
	///
	/// The returned Vec is sorted by frequency.
	///
	/// Note: performance-wise, FFT works better when the signal length is a power of two.
	///
	/// # Panics
	/// - if the passed `signal` is not compatible with the configured `samples_per_window`.
	#[must_use]
	pub fn analyze(&mut self, signal: &[f32]) -> &Vec<FftPoint> {
		let samples = signal.len();

		assert_eq!(
			samples, self.samples_per_window,
			"signal with incompatible length received"
		);

		for (i, c) in self.complex_signal.iter_mut().enumerate() {
			*c = Complex::new(
				signal[i] * (self.windowing_fn).ratio_at(i, self.samples_per_window),
				0.0,
			);
		}

		self.fft_processor.process(&mut self.complex_signal);

		// https://docs.rs/rustfft/6.2.0/rustfft/index.html#normalization
		#[allow(clippy::cast_precision_loss)]
		let normalization_factor = 1.0 / (samples as f32).sqrt();

		for (transform_i, complex_i) in self.frequency_indices.clone().enumerate() {
			self.cur_transform[transform_i] = FftPoint {
				frequency: index_to_frequency(complex_i, self.sample_rate, samples),
				magnitude: (self.complex_signal[complex_i] * normalization_factor).norm(),
			}
		}

		&self.cur_transform
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

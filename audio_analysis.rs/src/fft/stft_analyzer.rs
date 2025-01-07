use std::{ops::RangeInclusive, sync::Arc};

use rustfft::{num_complex::Complex, Fft, FftPlanner};

use crate::fft::index_to_frequency;

use super::{fft_frequency_bins, filtered_frequency_index_range, FftPoint, WindowingFn};

pub struct StftAnalyzer {
	sample_rate: usize,
	samples_per_window: usize,
	windowing_fn: Box<dyn WindowingFn + Send + 'static>,
	frequency_indices: RangeInclusive<usize>,
	fft_processor: Arc<dyn Fft<f32>>,
	complex_signal: Vec<Complex<f32>>,
	cur_transform: Vec<FftPoint>,
}

impl StftAnalyzer {
	pub fn new(
		sample_rate: usize,
		samples_per_window: usize,
		frequency_range: (f32, f32),
		windowing_fn: impl WindowingFn + Send + 'static,
	) -> Self {
		let mut planner = FftPlanner::new();
		let frequency_indices =
			filtered_frequency_index_range(sample_rate, samples_per_window, frequency_range);
		Self {
			sample_rate,
			samples_per_window,
			windowing_fn: Box::new(windowing_fn) as Box<dyn WindowingFn + Send + 'static>,

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

	///
	/// Analyze a signal in the domain of time, sampled at the configured sample rate.
	///
	/// The returned Vec is sorted by frequency.
	///
	/// Note: performance-wise, FFT works better when the signal length is a power of two.
	///
	/// # Panics
	/// -
	///
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
}

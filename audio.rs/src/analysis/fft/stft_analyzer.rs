use std::{ops::RangeInclusive, sync::Arc};

use rustfft::{num_complex::Complex, Fft, FftPlanner};

use crate::{analysis::{fft::FftBinPoint, WindowingFn}, NOfSamples};

use super::{fft_frequency_bins, filtered_frequency_index_range, FftPoint};

#[derive(Clone)]
pub struct StftAnalyzer<const SAMPLE_RATE: usize, const SAMPLES_PER_WINDOW: usize> {
	windowing_fn: Arc<dyn WindowingFn + Sync + Send + 'static>,
	frequency_indices: RangeInclusive<usize>,
	fft_processor: Arc<dyn Fft<f32>>,
	complex_signal: Vec<Complex<f32>>,
	cur_transform_bins: Vec<FftBinPoint<SAMPLE_RATE, SAMPLES_PER_WINDOW>>,
	cur_transform: Vec<FftPoint<SAMPLE_RATE, SAMPLES_PER_WINDOW>>,
}

impl<const SAMPLE_RATE: usize, const SAMPLES_PER_WINDOW: usize> std::fmt::Debug
	for StftAnalyzer<SAMPLE_RATE, SAMPLES_PER_WINDOW>
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct(&format!(
			"StftAnalyzer<{SAMPLE_RATE}, {SAMPLES_PER_WINDOW}>"
		))
		.field("windowing_fn", &"omitted")
		.field("frequency_indices", &self.frequency_indices)
		.field("fft_processor", &"omitted")
		.field("complex_signal", &self.complex_signal)
		.field("cur_transform_bins", &self.cur_transform_bins)
		.field("cur_transform", &self.cur_transform)
		.finish()
	}
}

impl<const SAMPLE_RATE: usize, const SAMPLES_PER_WINDOW: usize>
	StftAnalyzer<SAMPLE_RATE, SAMPLES_PER_WINDOW>
{
	#[must_use]
	pub fn new(
		frequency_range: (f32, f32),
		windowing_fn: impl WindowingFn + Send + Sync + 'static,
	) -> Self {
		let mut planner = FftPlanner::new();
		let frequency_indices =
			filtered_frequency_index_range(NOfSamples::<SAMPLE_RATE>::new(SAMPLES_PER_WINDOW), frequency_range);
		let transform_size = frequency_indices.clone().count();
		Self {
			windowing_fn: Arc::new(windowing_fn) as Arc<dyn WindowingFn + Send + Sync + 'static>,

			frequency_indices: frequency_indices.clone(),
			fft_processor: planner.plan_fft_forward(SAMPLES_PER_WINDOW),
			complex_signal: vec![Complex { re: 0., im: 0. }; SAMPLES_PER_WINDOW],
			cur_transform_bins: vec![
				FftBinPoint {
					magnitude: 0.,
					frequency_idx: 0
				};
				transform_size
			],
			cur_transform: vec![
				FftPoint {
					magnitude: 0.,
					frequency: 0.
				};
				transform_size
			],
		}
	}

	#[must_use]
	pub fn frequency_bins(&self) -> Vec<f32> {
		fft_frequency_bins(NOfSamples::<SAMPLE_RATE>::new(SAMPLES_PER_WINDOW))
			.skip(*self.frequency_indices.start())
			.take(self.frequency_indices.clone().count())
			.collect()
	}

	/// Analyze a signal in the domain of time, sampled at the configured sample rate.
	///
	/// The returned Vec is sorted by frequency bin.
	///
	/// Note: performance-wise, FFT works better when the signal length is a power of two.
	///
	/// # Panics
	/// - if the passed `signal` is not compatible with the configured `samples_per_window`.
	#[must_use]
	pub fn analyze_bins(&mut self, signal: &[f32]) -> &Vec<FftBinPoint<SAMPLE_RATE, SAMPLES_PER_WINDOW>> {
		let samples = signal.len();

		assert_eq!(
			samples, SAMPLES_PER_WINDOW,
			"signal with incompatible length received"
		);

		for (i, c) in self.complex_signal.iter_mut().enumerate() {
			*c = Complex::new(
				signal[i] * (self.windowing_fn).ratio_at(i, SAMPLES_PER_WINDOW),
				0.0,
			);
		}

		self.fft_processor.process(&mut self.complex_signal);

		// https://docs.rs/rustfft/6.2.0/rustfft/index.html#normalization
		#[allow(clippy::cast_precision_loss)]
		let normalization_factor = 1.0 / (samples as f32).sqrt();

		for (transform_i, complex_i) in self.frequency_indices.clone().enumerate() {
			self.cur_transform_bins[transform_i] = FftBinPoint {
				frequency_idx: complex_i,
				magnitude: (self.complex_signal[complex_i] * normalization_factor).norm(),
			}
		}

		&self.cur_transform_bins
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
	pub fn analyze(&mut self, signal: &[f32]) -> &Vec<FftPoint<SAMPLE_RATE,SAMPLES_PER_WINDOW>> {
		// update cur_transform_bins
		let _bin_transform = self.analyze_bins(signal);

		for (dst, src) in self
			.cur_transform
			.iter_mut()
			.zip(self.cur_transform_bins.iter())
		{
			*dst = src.to_fft_point();
		}
		&self.cur_transform
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

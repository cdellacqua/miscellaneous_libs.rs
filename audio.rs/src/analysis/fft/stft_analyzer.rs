use std::sync::Arc;

use rustfft::{num_complex::Complex, Fft, FftPlanner};

use crate::{
	analysis::{fft::FftBinPoint, windowing_fns::HannWindow, WindowingFn},
	NOfSamples,
};

use super::{fft_frequency_bins, fft_real_length, FftPoint};

#[derive(Clone)]
pub struct StftAnalyzer<const SAMPLE_RATE: usize, const SAMPLES_PER_WINDOW: usize> {
	windowing_fn: Arc<dyn WindowingFn + Sync + Send + 'static>,
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
	pub fn new(windowing_fn: impl WindowingFn + Send + Sync + 'static) -> Self {
		let mut planner = FftPlanner::new();
		let transform_size = fft_real_length(*NOfSamples::<SAMPLE_RATE>::new(SAMPLES_PER_WINDOW));
		Self {
			windowing_fn: Arc::new(windowing_fn) as Arc<dyn WindowingFn + Send + Sync + 'static>,

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
		fft_frequency_bins(NOfSamples::<SAMPLE_RATE>::new(SAMPLES_PER_WINDOW)).collect()
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
	pub fn analyze_bins(
		&mut self,
		signal: &[f32],
	) -> &Vec<FftBinPoint<SAMPLE_RATE, SAMPLES_PER_WINDOW>> {
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

		self.cur_transform_bins
			.iter_mut()
			.zip(self.complex_signal.iter())
			.enumerate()
			.for_each(|(i, (dst, src))| {
				*dst = FftBinPoint {
					frequency_idx: i,
					magnitude: (src * normalization_factor).norm(),
				};
			});

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
	pub fn analyze(&mut self, signal: &[f32]) -> &Vec<FftPoint<SAMPLE_RATE, SAMPLES_PER_WINDOW>> {
		// update cur_transform_bins
		let _bin_transform = self.analyze_bins(signal);

		self.cur_transform
			.iter_mut()
			.zip(self.cur_transform_bins.iter())
			.for_each(|(dst, src)| {
				*dst = src.to_fft_point();
			});

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

impl<const SAMPLE_RATE: usize, const SAMPLES_PER_WINDOW: usize> Default
	for StftAnalyzer<SAMPLE_RATE, SAMPLES_PER_WINDOW>
{
	fn default() -> Self {
		Self::new(HannWindow)
	}
}

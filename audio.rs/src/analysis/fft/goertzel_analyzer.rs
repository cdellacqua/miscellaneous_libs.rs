use std::{f32::consts::TAU, sync::Arc};

use rustfft::num_complex::Complex32;

use crate::analysis::WindowingFn;

use super::FftBinPoint;

#[derive(Clone)]
pub struct GoertzelAnalyzer<const SAMPLE_RATE: usize, const SAMPLES_PER_WINDOW: usize> {
	windowing_fn: Arc<dyn WindowingFn + Sync + Send + 'static>,
	cur_transform_bins: Vec<FftBinPoint<SAMPLE_RATE, SAMPLES_PER_WINDOW>>,
	frequency_bins: Vec<usize>,
	coefficients: Vec<(f32, Complex32)>,
}

impl<const SAMPLE_RATE: usize, const SAMPLES_PER_WINDOW: usize> std::fmt::Debug
	for GoertzelAnalyzer<SAMPLE_RATE, SAMPLES_PER_WINDOW>
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct(&format!(
			"GoertzelAnalyzer<{SAMPLE_RATE}, {SAMPLES_PER_WINDOW}>"
		))
		.field("windowing_fn", &"omitted")
		.field("cur_transform_bins", &self.cur_transform_bins)
		.field("frequency_bins", &self.frequency_bins)
		.field("coefficients", &self.coefficients)
		.finish()
	}
}

impl<const SAMPLE_RATE: usize, const SAMPLES_PER_WINDOW: usize>
	GoertzelAnalyzer<SAMPLE_RATE, SAMPLES_PER_WINDOW>
{
	#[allow(clippy::cast_precision_loss)]
	pub fn new(
		mut frequency_bins: Vec<usize>,
		windowing_fn: impl WindowingFn + Send + Sync + 'static,
	) -> Self {
		frequency_bins.sort_unstable();
		Self {
			// Pre-computing coefficients
			coefficients: frequency_bins
				.iter()
				.map(|&bin| {
					let ω = TAU * bin as f32 / SAMPLES_PER_WINDOW as f32;
					(2.0 * ω.cos(), Complex32::new(ω.cos(), ω.sin()))
				})
				.collect(),
			cur_transform_bins: vec![FftBinPoint::default(); frequency_bins.len()],
			frequency_bins,
			windowing_fn: Arc::new(windowing_fn),
		}
	}

	/// Analyze a signal in the domain of time, sampled at the configured sample rate.
	///
	/// The returned `Vec` is sorted by frequency bin.
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

		// Normalization also applies here.
		// https://docs.rs/rustfft/6.2.0/rustfft/index.html#normalization
		#[allow(clippy::cast_precision_loss)]
		let normalization_factor = 1.0 / (samples as f32).sqrt();

		let windowed_signal: Vec<f32> = signal
			.iter()
			.enumerate()
			.map(|(i, &sample)| sample * (self.windowing_fn).ratio_at(i, SAMPLES_PER_WINDOW))
			.collect();

		for ((&bin, coeff), bin_point) in self
			.frequency_bins
			.iter()
			.zip(self.coefficients.iter())
			.zip(self.cur_transform_bins.iter_mut())
		{
			let mut z1 = 0.0;
			let mut z2 = 0.0;

			for sample in &windowed_signal {
				let z0 = sample + coeff.0 * z1 - z2;
				z2 = z1;
				z1 = z0;
			}

			*bin_point = FftBinPoint {
				c: Complex32::new(z1 * coeff.1.re - z2, z1 * coeff.1.im) * normalization_factor,
				bin_idx: bin,
			};
		}

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

use std::sync::Arc;

use rustfft::{
	num_complex::{Complex, Complex32},
	Fft, FftPlanner,
};

use crate::analysis::{DftCtx, DiscreteHarmonic, WindowingFn};

#[derive(Clone)]
pub struct StftAnalyzer {
	dft_ctx: DftCtx,
	windowing_values: Vec<f32>,
	fft_processor: Arc<dyn Fft<f32>>,
	complex_signal: Vec<Complex32>,
	cur_transform: Vec<DiscreteHarmonic>,
	normalization_factor: f32,
}

impl std::fmt::Debug for StftAnalyzer {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("StftAnalyzer")
			.field("dft_ctx", &self.dft_ctx)
			.field("windowing_values", &self.windowing_values)
			.field("fft_processor", &"omitted")
			.field("complex_signal", &self.complex_signal)
			.field("cur_transform", &self.cur_transform)
			.field("normalization_factor", &self.normalization_factor)
			.finish()
	}
}

impl StftAnalyzer {
	#[must_use]
	pub fn new(dft_ctx: DftCtx, windowing_fn: &impl WindowingFn) -> Self {
		let mut planner = FftPlanner::new();
		let transform_size = dft_ctx.n_of_bins();
		Self {
			dft_ctx,
			windowing_values: (0..dft_ctx.samples_per_window())
				.map(|i| windowing_fn.ratio_at(i, dft_ctx.samples_per_window()))
				.collect(),
			fft_processor: planner.plan_fft_forward(dft_ctx.samples_per_window()),
			complex_signal: vec![Complex { re: 0., im: 0. }; dft_ctx.samples_per_window()],
			cur_transform: (0..transform_size)
				.map(|i| DiscreteHarmonic::new(Complex::ZERO, i))
				.collect(),
			// https://docs.rs/rustfft/6.2.0/rustfft/index.html#normalization
			#[allow(clippy::cast_precision_loss)]
			normalization_factor: 1.0 / (dft_ctx.samples_per_window() as f32).sqrt(),
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
			samples,
			self.dft_ctx.samples_per_window(),
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

		let transform_size = self.cur_transform.len();
		self.cur_transform
			.iter_mut()
			.zip(self.complex_signal.iter().take(transform_size))
			.for_each(|(dst, src)| {
				dst.phasor = src * self.normalization_factor;
			});

		&self.cur_transform
	}

	#[must_use]
	pub fn dft_ctx(&self) -> DftCtx {
		self.dft_ctx
	}
}

#[cfg(test)]
#[cfg(feature = "output")]
mod tests {
	use math_utils::one_dimensional_mapping::MapRatio;

	use crate::{
		analysis::{windowing_fns::HannWindow, Harmonic},
		output::harmonics_to_samples,
		SampleRate,
	};

	use super::*;

	#[test]
	#[allow(clippy::cast_precision_loss)]
	fn stft_peaks_at_frequency_bin() {
		let dft_ctx = DftCtx::new(SampleRate(44100), 44100);

		let mut stft_analyzer = StftAnalyzer::new(dft_ctx, &HannWindow);
		let bins = dft_ctx.bins();
		let delta_hz = dft_ctx.bin_to_frequency(bins[1]) - dft_ctx.bin_to_frequency(bins[0]);

		for i in 1..100 {
			let frequency = (i as f32 / 100.0).map_ratio((
				dft_ctx.bin_to_frequency(bins[10]) - delta_hz / 2.,
				dft_ctx.bin_to_frequency(bins[10]) + delta_hz / 2.,
			));

			let signal = harmonics_to_samples(
				dft_ctx.sample_rate(),
				dft_ctx.samples_per_window(),
				&[Harmonic::new(Complex32::ONE, frequency)],
			);
			let analysis = stft_analyzer.analyze(&signal);
			assert_eq!(
				analysis
					.iter()
					.max_by(|a, b| a.power().total_cmp(&b.power()))
					.unwrap()
					.bin(),
				bins[10]
			);
		}
	}

	#[test]
	#[allow(clippy::cast_precision_loss)]
	fn stft_peaks_at_frequency_bin_440() {
		let dft_ctx = DftCtx::new(SampleRate(44100), 100);

		let mut stft_analyzer = StftAnalyzer::new(dft_ctx, &HannWindow);
		let signal = harmonics_to_samples(
			dft_ctx.sample_rate(),
			dft_ctx.samples_per_window(),
			&[Harmonic::new(Complex32::ONE, 440.)],
		);
		let analysis = stft_analyzer.analyze(&signal);
		let h = analysis[1..] // skip 0Hz
			.iter()
			.max_by(|a, b| a.power().total_cmp(&b.power()))
			.unwrap();
		assert_eq!(h.bin(), 1);
		assert!(h.phase().abs() < 0.01);
	}

	#[test]
	#[allow(clippy::cast_precision_loss)]
	fn stft_phase() {
		let dft_ctx = DftCtx::new(SampleRate(44100), 4410);

		let bin = 50;

		let mut stft_analyzer = StftAnalyzer::new(dft_ctx, &HannWindow);

		let frequency = dft_ctx.bin_to_frequency(bin);

		let signal = harmonics_to_samples(
			dft_ctx.sample_rate(),
			dft_ctx.samples_per_window(),
			&[Harmonic::new(Complex32::ONE, frequency)],
		);
		let analysis = stft_analyzer.analyze(&signal);
		let phase = analysis
			.iter()
			.max_by(|a, b| a.power().total_cmp(&b.power()))
			.unwrap()
			.phase();
		assert!(phase.abs() < 0.001, "{phase}");
	}
}

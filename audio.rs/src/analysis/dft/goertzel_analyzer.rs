use std::f32::consts::TAU;

use rustfft::num_complex::{Complex, Complex32};

use crate::analysis::{DftCtx, DiscreteHarmonic, WindowingFn};

#[derive(Debug)]
pub struct GoertzelAnalyzer {
	dft_ctx: DftCtx,
	windowing_values: Vec<f32>,
	cur_transform: Vec<DiscreteHarmonic>,
	cur_signal: Vec<f32>,
	coefficients: Vec<(f32, Complex32)>,
	normalization_factor: f32,
}

impl GoertzelAnalyzer {
	#[allow(clippy::cast_precision_loss)]
	pub fn new(
		dft_ctx: DftCtx,
		mut frequency_bins: Vec<usize>,
		windowing_fn: &impl WindowingFn,
	) -> Self {
		frequency_bins.sort_unstable();
		Self {
			dft_ctx,
			// Pre-computing coefficients
			coefficients: frequency_bins
				.iter()
				.map(|&bin| {
					let ω = TAU * bin as f32 / dft_ctx.samples_per_window() as f32;
					(2.0 * ω.cos(), Complex32::new(ω.cos(), ω.sin()))
				})
				.collect(),
			cur_transform: frequency_bins
				.into_iter()
				.map(|bin| DiscreteHarmonic::new(Complex::ZERO, bin))
				.collect(),
			cur_signal: vec![0.; dft_ctx.samples_per_window()],
			windowing_values: (0..dft_ctx.samples_per_window())
				.map(|i| windowing_fn.ratio_at(i, dft_ctx.samples_per_window()))
				.collect(),
			// Normalization also applies here.
			// https://docs.rs/rustfft/6.2.0/rustfft/index.html#normalization
			#[allow(clippy::cast_precision_loss)]
			normalization_factor: 1.0 / (dft_ctx.samples_per_window() as f32).sqrt(),
		}
	}

	/// Analyze a signal in the domain of time, sampled at the configured sample rate.
	///
	/// The returned `Vec` is sorted by frequency bin.
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

		for ((dst, sample), windowing_value) in self
			.cur_signal
			.iter_mut()
			.zip(signal)
			.zip(self.windowing_values.iter())
		{
			*dst = sample * windowing_value;
		}

		for (coeff, bin_point) in self.coefficients.iter().zip(self.cur_transform.iter_mut()) {
			let mut z1 = 0.0;
			let mut z2 = 0.0;

			for &sample in &self.cur_signal {
				let z0 = sample + coeff.0 * z1 - z2;
				z2 = z1;
				z1 = z0;
			}

			bin_point.phasor =
				Complex32::new(z1 * coeff.1.re - z2, z1 * coeff.1.im) * self.normalization_factor;
		}

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
	use super::*;
	use crate::{
		analysis::{windowing_fns::HannWindow, Harmonic},
		output::harmonics_to_samples,
		SampleRate,
	};
	use math_utils::one_dimensional_mapping::MapRatio;

	#[test]
	#[allow(clippy::cast_precision_loss)]
	fn goertzel_peaks_at_frequency_bin() {
		let dft_ctx = DftCtx::new(crate::SampleRate(44100), 4410);

		let bin = 50;

		let mut stft_analyzer = GoertzelAnalyzer::new(
			dft_ctx,
			vec![bin - 2, bin - 1, bin, bin + 1, bin + 2],
			&HannWindow,
		);

		for i in 1..100 {
			let frequency = (i as f32 / 100.).map_ratio(dft_ctx.bin_frequency_interval(bin));

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
				bin
			);
		}
	}

	#[test]
	#[allow(clippy::cast_precision_loss)]
	fn goertzel_phase() {
		let dft_ctx = DftCtx::new(SampleRate(44100), 4410);

		let bin = 50;

		let mut stft_analyzer = GoertzelAnalyzer::new(
			dft_ctx,
			vec![bin - 2, bin - 1, bin, bin + 1, bin + 2],
			&HannWindow,
		);

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

	#[test]
	#[allow(clippy::cast_precision_loss)]
	fn goertzel_peaks_at_frequency_bin_440() {
		let dft_ctx = DftCtx::new(SampleRate(44100), 100);

		let bin = dft_ctx.frequency_to_bin(441.);
		assert_eq!(bin, 1);
		let mut stft_analyzer =
			GoertzelAnalyzer::new(dft_ctx, vec![bin, bin + 1, bin + 2], &HannWindow);
		let signal = harmonics_to_samples(
			dft_ctx.sample_rate(),
			dft_ctx.samples_per_window(),
			&[Harmonic::new(Complex32::ONE, 440.)],
		);
		let analysis = stft_analyzer.analyze(&signal);
		let h = analysis
			.iter()
			.max_by(|a, b| a.power().total_cmp(&b.power()))
			.unwrap();
		assert_eq!(h.bin(), 1);
		assert!(h.phase().abs() < 0.01);
	}
}

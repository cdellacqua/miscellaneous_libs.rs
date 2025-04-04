use std::f32::consts::TAU;

use rustfft::num_complex::{Complex, Complex32};

use crate::analysis::{DiscreteHarmonic, DiscreteFrequency, WindowingFn};

#[derive(Debug)]
pub struct GoertzelAnalyzer {
	sample_rate: usize,
	samples_per_window: usize,
	windowing_values: Vec<f32>,
	cur_transform: Vec<DiscreteHarmonic>,
	cur_signal: Vec<f32>,
	coefficients: Vec<(f32, Complex32)>,
	normalization_factor: f32,
}

impl GoertzelAnalyzer {
	#[allow(clippy::cast_precision_loss)]
	pub fn new(
		sample_rate: usize,
		samples_per_window: usize,
		mut frequency_bins: Vec<DiscreteFrequency>,
		windowing_fn: &impl WindowingFn,
	) -> Self {
		frequency_bins.sort_unstable();
		Self {
			sample_rate,
			samples_per_window,
			// Pre-computing coefficients
			coefficients: frequency_bins
				.iter()
				.map(|&bin| {
					let ω = TAU * bin.bin_idx() as f32 / samples_per_window as f32;
					(2.0 * ω.cos(), Complex32::new(ω.cos(), ω.sin()))
				})
				.collect(),
			cur_transform: frequency_bins
				.into_iter()
				.map(|bin| {
					DiscreteHarmonic::new(sample_rate, samples_per_window, Complex::ZERO, bin)
				})
				.collect(),
			cur_signal: vec![0.; samples_per_window],
			windowing_values: (0..samples_per_window)
				.map(|i| windowing_fn.ratio_at(i, samples_per_window))
				.collect(),
			// Normalization also applies here.
			// https://docs.rs/rustfft/6.2.0/rustfft/index.html#normalization
			#[allow(clippy::cast_precision_loss)]
			normalization_factor: 1.0 / (samples_per_window as f32).sqrt(),
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
			samples, self.samples_per_window,
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
	use super::*;
	use crate::{
		analysis::{windowing_fns::HannWindow, Harmonic},
		output::harmonics_to_samples,
	};
	use math_utils::one_dimensional_mapping::MapRatio;

	#[test]
	#[allow(clippy::cast_precision_loss)]
	fn goertzel_peaks_at_frequency_bin() {
		const SAMPLE_RATE: usize = 44100;
		const SAMPLES_PER_WINDOW: usize = 4410;

		let bin = DiscreteFrequency::new(SAMPLE_RATE, SAMPLES_PER_WINDOW, 50);

		let mut stft_analyzer = GoertzelAnalyzer::new(
			SAMPLE_RATE,
			SAMPLES_PER_WINDOW,
			vec![bin - 2, bin - 1, bin, bin + 1, bin + 2],
			&HannWindow,
		);

		for i in 1..100 {
			let frequency = (i as f32 / 100.).map_ratio(bin.frequency_interval());

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
					.frequency() - bin.frequency())
				.abs() < f32::EPSILON,
				"{frequency} {}",
				bin.frequency()
			);
		}
	}

	#[test]
	#[allow(clippy::cast_precision_loss)]
	fn goertzel_phase() {
		const SAMPLE_RATE: usize = 44100;
		const SAMPLES_PER_WINDOW: usize = 4410;

		let bin = DiscreteFrequency::new(SAMPLE_RATE, SAMPLES_PER_WINDOW, 50);

		let mut stft_analyzer = GoertzelAnalyzer::new(
			SAMPLE_RATE,
			SAMPLES_PER_WINDOW,
			vec![bin - 2, bin - 1, bin, bin + 1, bin + 2],
			&HannWindow,
		);

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

	#[test]
	#[allow(clippy::cast_precision_loss)]
	fn goertzel_peaks_at_frequency_bin_440() {
		const SAMPLE_RATE: usize = 44100;
		const SAMPLES_PER_WINDOW: usize = 100;

		let bin = DiscreteFrequency::from_frequency(SAMPLE_RATE, SAMPLES_PER_WINDOW, 441.);
		assert_eq!(bin.bin_idx(), 1);
		let mut stft_analyzer = GoertzelAnalyzer::new(
			SAMPLE_RATE,
			SAMPLES_PER_WINDOW,
			vec![bin, bin + 1, bin + 2],
			&HannWindow,
		);
		let signal = harmonics_to_samples::<SAMPLE_RATE>(
			SAMPLES_PER_WINDOW,
			&[Harmonic::new(Complex32::ONE, 440.)],
		);
		let analysis = stft_analyzer.analyze(signal.as_mono());
		let harmonic = analysis
			.iter()
			.max_by(|a, b| a.power().total_cmp(&b.power()))
			.unwrap();
		assert_eq!(harmonic.bin_idx(), 1);
		assert!(harmonic.phase().abs() < 0.01);
	}
}

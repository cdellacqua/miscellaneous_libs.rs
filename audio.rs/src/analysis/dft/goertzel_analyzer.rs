use std::f32::consts::TAU;

use rustfft::num_complex::Complex32;

use crate::analysis::{FrequencyBin, Harmonic, WindowingFn};

#[derive(Debug)]
pub struct GoertzelAnalyzer<const SAMPLE_RATE: usize, const SAMPLES_PER_WINDOW: usize> {
	windowing_values: Vec<f32>,
	cur_transform: Vec<Harmonic<SAMPLE_RATE, SAMPLES_PER_WINDOW>>,
	cur_signal: Vec<f32>,
	frequency_bins: Vec<FrequencyBin<SAMPLE_RATE, SAMPLES_PER_WINDOW>>,
	coefficients: Vec<(f32, Complex32)>,
	normalization_factor: f32,
}

impl<const SAMPLE_RATE: usize, const SAMPLES_PER_WINDOW: usize>
	GoertzelAnalyzer<SAMPLE_RATE, SAMPLES_PER_WINDOW>
{
	#[allow(clippy::cast_precision_loss)]
	pub fn new(
		mut frequency_bins: Vec<FrequencyBin<SAMPLE_RATE, SAMPLES_PER_WINDOW>>,
		windowing_fn: &impl WindowingFn,
	) -> Self {
		frequency_bins.sort_unstable();
		Self {
			// Pre-computing coefficients
			coefficients: frequency_bins
				.iter()
				.map(|&bin| {
					let ω = TAU * bin.bin_idx() as f32 / SAMPLES_PER_WINDOW as f32;
					(2.0 * ω.cos(), Complex32::new(ω.cos(), ω.sin()))
				})
				.collect(),
			cur_transform: vec![Harmonic::default(); frequency_bins.len()],
			cur_signal: vec![0.; SAMPLES_PER_WINDOW],
			frequency_bins,
			windowing_values: (0..SAMPLES_PER_WINDOW)
				.map(|i| windowing_fn.ratio_at(i, SAMPLES_PER_WINDOW))
				.collect(),
			// Normalization also applies here.
			// https://docs.rs/rustfft/6.2.0/rustfft/index.html#normalization
			#[allow(clippy::cast_precision_loss)]
			normalization_factor: 1.0 / (SAMPLES_PER_WINDOW as f32).sqrt(),
		}
	}

	/// Analyze a signal in the domain of time, sampled at the configured sample rate.
	///
	/// The returned `Vec` is sorted by frequency bin.
	///
	/// # Panics
	/// - if the passed `signal` is not compatible with the configured `samples_per_window`.
	#[must_use]
	pub fn analyze(&mut self, signal: &[f32]) -> &Vec<Harmonic<SAMPLE_RATE, SAMPLES_PER_WINDOW>> {
		let samples = signal.len();

		assert_eq!(
			samples, SAMPLES_PER_WINDOW,
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

		for ((&bin, coeff), bin_point) in self
			.frequency_bins
			.iter()
			.zip(self.coefficients.iter())
			.zip(self.cur_transform.iter_mut())
		{
			let mut z1 = 0.0;
			let mut z2 = 0.0;

			for &sample in &self.cur_signal {
				let z0 = sample + coeff.0 * z1 - z2;
				z2 = z1;
				z1 = z0;
			}

			*bin_point = Harmonic::new(
				Complex32::new(z1 * coeff.1.re - z2, z1 * coeff.1.im) * self.normalization_factor,
				bin,
			);
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

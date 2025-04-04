#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use std::{
	f32::consts::TAU,
	sync::{Arc, Mutex},
	time::Duration,
};

use mutex_ext::LockExt;

use crate::{
	analysis::Harmonic, AudioStreamBuilderError, AudioStreamSamplingState, NOfFrames, SampleRate,
	SamplingCtx,
};

use super::OutputStream;

struct OscillatorState {
	frame_idx: NOfFrames,
	harmonics: Vec<Harmonic>,
	mute: bool,
}

pub struct Oscillator {
	shared: Arc<Mutex<OscillatorState>>,
	base_stream: OutputStream,
}

impl Oscillator {
	/// Build and start sampling an input stream
	///
	/// # Errors
	/// [`AudioStreamBuilderError`]
	pub fn new(
		sampling_ctx: SamplingCtx,
		device_name: Option<&str>,
	) -> Result<Self, AudioStreamBuilderError> {
		let shared = Arc::new(Mutex::new(OscillatorState {
			frame_idx: NOfFrames(0),
			mute: false,
			harmonics: vec![],
		}));

		let base_stream = OutputStream::new(
			sampling_ctx,
			device_name,
			Box::new({
				let shared = shared.clone();
				move |mut chunk| {
					shared.with_lock_mut(|shared| {
						if shared.mute {
							chunk.raw_buffer_mut().fill(0.);
						} else {
							let harmonics = &shared.harmonics;

							let harmonics_data: Vec<_> = harmonics
								.iter()
								.map(|h| (h.amplitude(), h.phase(), h.frequency()))
								.collect();

							let sum_of_amplitudes = harmonics_data
								.iter()
								.map(|(amplitude, ..)| amplitude)
								.sum::<f32>();

							for i in 0..chunk.n_of_frames().0 {
								chunk.at_mut(i).samples_mut().fill(
									harmonics_data
										.iter()
										.map(|(amplitude, phase, frequency)| {
											amplitude / sum_of_amplitudes
												* f32::cos(
													phase
														+ TAU
															* frequency * ((shared.frame_idx.0 + i)
															as f32 / sampling_ctx
															.sample_rate()
															.0
															as f32),
												)
										})
										.sum::<f32>(),
								);
							}

							shared.frame_idx += chunk.n_of_frames();
						}
					});
				}
			}),
			None,
		)?;
		Ok(Self {
			shared,
			base_stream,
		})
	}

	#[must_use]
	pub fn state(&self) -> AudioStreamSamplingState {
		self.base_stream.state()
	}

	/// # Panics
	/// - if the mutex guarding the internal state is poisoned.
	pub fn set_harmonics(&mut self, harmonics: Vec<Harmonic>) {
		self.shared.with_lock_mut(|shared| {
			shared.harmonics = harmonics;
			shared.frame_idx = NOfFrames(0);
		});
	}

	/// # Panics
	/// - if the mutex guarding the internal state is poisoned.
	#[must_use]
	pub fn harmonics(&self) -> Vec<Harmonic> {
		self.shared.with_lock(|shared| shared.harmonics.clone())
	}

	/// # Panics
	/// - if the mutex guarding the internal state is poisoned.
	pub fn set_mute(&mut self, mute: bool) {
		self.shared.with_lock_mut(|shared| {
			shared.mute = mute;
		});
	}

	/// # Panics
	/// - if the mutex guarding the internal state is poisoned.
	#[must_use]
	pub fn mute(&self) -> bool {
		self.shared.with_lock(|shared| shared.mute)
	}

	#[must_use]
	pub fn sampling_ctx(&self) -> SamplingCtx {
		self.base_stream.sampling_ctx()
	}

	#[must_use]
	pub fn sample_rate(&self) -> SampleRate {
		self.base_stream.sample_rate()
	}

	#[must_use]
	pub fn n_ch(&self) -> usize {
		self.base_stream.n_ch()
	}

	#[must_use]
	pub fn avg_output_delay(&self) -> Duration {
		self.base_stream.avg_output_delay()
	}
}

/// Generate a series of samples computed using a cosine wave with the
/// specified frequency, phase and amplitude.
#[must_use]
pub fn harmonics_to_samples(
	sample_rate: SampleRate,
	n_of_samples: usize,
	harmonics: &[Harmonic],
) -> Vec<f32> {
	// precompute all constants
	let harmonics_data: Vec<_> = harmonics
		.iter()
		.map(|h| (h.amplitude(), h.phase(), h.frequency()))
		.collect();

	let mut mono = (0..n_of_samples)
		.map(move |i| {
			#[allow(clippy::cast_precision_loss)]
			harmonics_data
				.iter()
				.map(|(amplitude, phase, frequency)| {
					amplitude
						* f32::cos(phase + TAU * frequency * (i as f32 / sample_rate.0 as f32))
				})
				.sum::<f32>()
		})
		.collect::<Vec<f32>>();

	let abs_max = mono
		.iter()
		.map(|s| s.abs())
		.max_by(f32::total_cmp)
		.unwrap_or(1.);

	mono.iter_mut().for_each(|s| *s /= abs_max);

	mono
}

#[cfg(test)]
mod tests {
	use std::{f32::consts::PI, thread::sleep, time::Duration};

	use math_utils::one_dimensional_mapping::MapRange;
	use rustfft::num_complex::Complex32;

	use crate::analysis::{dft::GoertzelAnalyzer, windowing_fns::HannWindow, DftCtx};

	use super::*;

	#[test]
	#[ignore = "manually run this test to hear to the resulting sound"]
	fn test_440() {
		let mut oscillator = Oscillator::new(SamplingCtx::new(SampleRate(44100), 1), None).unwrap();
		oscillator.set_harmonics(vec![Harmonic::new(Complex32::ONE, 440.)]);

		sleep(Duration::from_secs(10));
		assert!(!oscillator.mute());
	}
	#[test]
	#[ignore = "manually run this test to hear to the resulting sound"]
	fn test_440_333() {
		let mut oscillator = Oscillator::new(SamplingCtx::new(SampleRate(44100), 1), None).unwrap();
		oscillator.set_harmonics(vec![
			Harmonic::new(Complex32::ONE, 440.),
			Harmonic::new(Complex32::ONE, 333.),
		]);
		sleep(Duration::from_secs(10));
	}

	#[test]
	fn test_frequencies_to_samples() {
		let samples = harmonics_to_samples(
			SampleRate(44100),
			100,
			&[Harmonic::new(Complex32::ONE, 440.)],
		);
		assert!((samples[0] - 1.0).abs() < f32::EPSILON);
		assert!((samples[1] - 1.0).abs() > f32::EPSILON);
	}

	#[test]
	#[ignore = "manually run this test to check the phase of the output with a spectrum analyzer"]
	fn test_phase() {
		let mut oscillator = Oscillator::new(SamplingCtx::new(SampleRate(8000), 1), None).unwrap();
		oscillator.set_harmonics(vec![Harmonic::new(Complex32::from_polar(1., 0.), 2000.0)]);
		sleep(Duration::from_secs(50));
		oscillator.set_harmonics(vec![Harmonic::new(Complex32::from_polar(1., PI), 2000.0)]);
		sleep(Duration::from_secs(5));
	}

	#[test]
	fn test_harmonic_phases() {
		const N: usize = 10;
		const SAMPLE_RATE: SampleRate = SampleRate(44100);
		const SAMPLES_PER_WINDOW: usize = 64;
		let dft_ctx = DftCtx::new(SAMPLE_RATE, SAMPLES_PER_WINDOW);

		let bin = dft_ctx.frequency_to_bin(3000.0);
		for phase_idx in 0..100 {
			let ref_phase = (phase_idx as f32).map((0., 99.), (-PI, PI - 0.001));
			let impulse = harmonics_to_samples(
				SAMPLE_RATE,
				SAMPLES_PER_WINDOW,
				&[Harmonic::new(
					Complex32::from_polar(1., ref_phase),
					dft_ctx.bin_to_frequency(bin),
				)],
			);

			let mut signal = vec![0.; impulse.len() * N];
			for i in 0..N {
				signal[i * impulse.len()..(i + 1) * impulse.len()].copy_from_slice(&impulse);
			}
			let mut goertzel = GoertzelAnalyzer::new(dft_ctx, vec![bin], &HannWindow);
			let h = goertzel
				.analyze(&signal[0..impulse.len()])
				.first()
				.copied()
				.unwrap();

			let phase = h.phase();

			assert!(
				(phase - ref_phase).abs() < 0.001,
				"phase: {phase} - ref_phase: {ref_phase}"
			);

			for i in 1..N {
				let h = goertzel
					.analyze(&signal[i * impulse.len()..(i + 1) * impulse.len()])
					.first()
					.copied()
					.unwrap();
				let phase = h.phase();
				assert!(
					(phase - ref_phase).abs() < 0.001,
					"phase: {phase} - ref_phase: {ref_phase}"
				);
			}
		}
	}
}

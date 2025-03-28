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
	analysis::Harmonic, buffers::InterleavedAudioBuffer, AudioStreamBuilderError,
	AudioStreamSamplingState, NOfFrames,
};

use super::{OutputStream, OutputStreamBuilder};

/* TODO: support different set of frequencies per channel? */
#[derive(Debug, Clone, PartialEq)]
pub struct OscillatorBuilder<const SAMPLE_RATE: usize, const N_CH: usize> {
	harmonics: Vec<Harmonic>,
	mute: bool,
	device_name: Option<String>,
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> Default for OscillatorBuilder<SAMPLE_RATE, N_CH> {
	fn default() -> Self {
		Self::new(vec![], false, None)
	}
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> OscillatorBuilder<SAMPLE_RATE, N_CH> {
	#[must_use]
	pub fn new(harmonics: Vec<Harmonic>, mute: bool, device_name: Option<String>) -> Self {
		Self {
			harmonics,
			mute,
			device_name,
		}
	}

	/// Build and start output stream
	///
	/// # Errors
	/// [`AudioStreamBuilderError`]
	pub fn build(&self) -> Result<Oscillator<SAMPLE_RATE, N_CH>, AudioStreamBuilderError> {
		let shared = Arc::new(Mutex::new(OscillatorState {
			frame_idx: NOfFrames::new(0),
			signal: harmonics_to_samples(SAMPLE_RATE, &self.harmonics).multiply(),
			mute: false,
			harmonics: self.harmonics.clone(),
		}));

		Ok(Oscillator::new(
			shared.clone(),
			OutputStreamBuilder::new(
				self.device_name.clone(),
				Box::new(move |mut chunk| {
					let output_frames = chunk.n_of_frames();
					shared.with_lock_mut(|shared| {
						if shared.mute {
							chunk.raw_buffer_mut().fill(0.);
						} else {
							let signal = &shared.signal;

							let mut output_idx = NOfFrames::new(0);
							while output_idx < output_frames {
								let frame_idx_mod: NOfFrames<SAMPLE_RATE, N_CH> =
									shared.frame_idx % signal.n_of_frames().inner();
								let available = (chunk.n_of_frames() - output_idx)
									.min(signal.n_of_frames() - frame_idx_mod);

								chunk.raw_buffer_mut()[output_idx.n_of_samples()
									..(output_idx + available).n_of_samples()]
									.copy_from_slice(
										&signal.raw_buffer()[frame_idx_mod.n_of_samples()
											..(frame_idx_mod + available).n_of_samples()],
									);
								output_idx += available;
								shared.frame_idx += available;
							}
						}
					});
				}),
				None,
			)
			.build()?,
		))
	}
}

struct OscillatorState<const SAMPLE_RATE: usize, const N_CH: usize> {
	frame_idx: NOfFrames<SAMPLE_RATE, N_CH>,
	signal: InterleavedAudioBuffer<SAMPLE_RATE, N_CH, Vec<f32>>,
	harmonics: Vec<Harmonic>,
	mute: bool,
}

pub struct Oscillator<const SAMPLE_RATE: usize, const N_CH: usize> {
	shared: Arc<Mutex<OscillatorState<SAMPLE_RATE, N_CH>>>,
	base_stream: OutputStream<SAMPLE_RATE, N_CH>,
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> Oscillator<SAMPLE_RATE, N_CH> {
	fn new(
		shared: Arc<Mutex<OscillatorState<SAMPLE_RATE, N_CH>>>,
		base_stream: OutputStream<SAMPLE_RATE, N_CH>,
	) -> Self {
		Self {
			shared,
			base_stream,
		}
	}

	#[must_use]
	pub fn state(&self) -> AudioStreamSamplingState {
		self.base_stream.state()
	}

	/// # Panics
	/// - if the mutex guarding the internal state is poisoned.
	pub fn set_harmonics(&mut self, harmonics: Vec<Harmonic>) {
		self.shared.with_lock_mut(|shared| {
			shared.signal = harmonics_to_samples::<SAMPLE_RATE>(SAMPLE_RATE, &harmonics).multiply();
			shared.harmonics = harmonics;
			shared.frame_idx = 0.into();
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
	pub fn sample_rate(&self) -> usize {
		SAMPLE_RATE
	}

	#[must_use]
	pub fn n_of_channels(&self) -> usize {
		N_CH
	}

	#[must_use]
	pub fn avg_output_delay(&self) -> Duration {
		self.base_stream.avg_output_delay()
	}
}

/// Generate a series of samples computed using a cosine wave with the
/// specified frequency, phase and amplitude.
#[must_use]
pub fn harmonics_to_samples<const SAMPLE_RATE: usize>(
	n_of_samples: usize,
	harmonics: &[Harmonic],
) -> InterleavedAudioBuffer<SAMPLE_RATE, 1, Vec<f32>> {
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
					amplitude * f32::cos(phase + TAU * frequency * (i as f32 / SAMPLE_RATE as f32))
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

	InterleavedAudioBuffer::new(mono)
}

#[cfg(test)]
mod tests {
	use std::{thread::sleep, time::Duration};

	use rustfft::num_complex::Complex32;

	use super::*;

	#[test]
	#[ignore = "manually run this test to hear to the resulting sound"]
	fn test_440() {
		let oscillator = OscillatorBuilder::<44100, 1>::new(
			vec![Harmonic::new(Complex32::ONE, 440.)],
			false,
			None,
		)
		.build()
		.unwrap();
		sleep(Duration::from_secs(10));
		assert!(!oscillator.mute());
	}
	#[test]
	#[ignore = "manually run this test to hear to the resulting sound"]
	fn test_440_333() {
		let _oscillator = OscillatorBuilder::<44100, 1>::new(
			vec![
				Harmonic::new(Complex32::ONE, 440.),
				Harmonic::new(Complex32::ONE, 333.),
			],
			false,
			None,
		)
		.build()
		.unwrap();
		sleep(Duration::from_secs(10));
	}

	#[test]
	fn test_frequencies_to_samples() {
		let samples = harmonics_to_samples::<44100>(100, &[Harmonic::new(Complex32::ONE, 440.)]);
		assert!((samples.as_mono()[0] - 1.0).abs() < f32::EPSILON);
		assert!((samples.as_mono()[1] - 1.0).abs() > f32::EPSILON);
	}
}

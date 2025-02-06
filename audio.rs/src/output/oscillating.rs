#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use std::{
	f32::consts::TAU,
	sync::{Arc, RwLock},
};

use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Device, SampleFormat, SampleRate, Stream, SupportedStreamConfig,
};
use resource_daemon::ResourceDaemon;

use crate::{
	buffers::InterleavedAudioBuffer, AudioStreamBuilderError, AudioStreamError,
	AudioStreamSamplingState, NOfSamples,
};

/* TODO: support different set of frequencies per channel? */
#[derive(Debug, Clone)]
pub struct OscillatorBuilder<const SAMPLE_RATE: usize, const N_CH: usize> {
	frequencies: Vec<f32>,
	mute: bool,
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> Default for OscillatorBuilder<SAMPLE_RATE, N_CH> {
	fn default() -> Self {
		Self::new(&[], false)
	}
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> OscillatorBuilder<SAMPLE_RATE, N_CH> {
	#[must_use]
	pub fn new(frequencies: &[f32], mute: bool) -> Self {
		Self {
			frequencies: frequencies.to_vec(),
			mute,
		}
	}

	/// Build and start output stream
	///
	/// # Errors
	/// [`AudioStreamBuilderError`]
	///
	/// # Panics
	/// - if the output device default configuration doesn't use f32 as the sample format.
	pub fn build(&self) -> Result<Oscillator<SAMPLE_RATE, N_CH>, AudioStreamBuilderError> {
		let device = cpal::default_host()
			.output_devices()
			.map_err(|_| AudioStreamBuilderError::UnableToListDevices)?
			.next()
			.ok_or(AudioStreamBuilderError::NoDeviceFound)?;

		let config = device
			.supported_input_configs()
			.map_err(|_| AudioStreamBuilderError::NoConfigFound)?
			.find(|c| c.channels() as usize == N_CH && c.sample_format() == SampleFormat::F32)
			.ok_or(AudioStreamBuilderError::NoConfigFound)?
			.try_with_sample_rate(SampleRate(SAMPLE_RATE as u32))
			.ok_or(AudioStreamBuilderError::NoConfigFound)?;

		// TODO: normalize everything to f32 and accept any format?
		assert!(
			matches!(config.sample_format(), cpal::SampleFormat::F32),
			"expected F32 input stream"
		);

		Ok(Oscillator::new(
			device,
			config,
			self.frequencies.clone(),
			self.mute,
		))
	}
}

struct OscillatorShared<const SAMPLE_RATE: usize, const N_CH: usize> {
	signal: InterleavedAudioBuffer<SAMPLE_RATE, N_CH, Vec<f32>>,
	frequencies: Vec<f32>,
	mute: bool,
}

pub struct Oscillator<const SAMPLE_RATE: usize, const N_CH: usize> {
	shared: Arc<RwLock<OscillatorShared<SAMPLE_RATE, N_CH>>>,
	stream_daemon: ResourceDaemon<Stream, AudioStreamError>,
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> Oscillator<SAMPLE_RATE, N_CH> {
	fn new(
		device: Device,
		config: SupportedStreamConfig,
		frequencies: Vec<f32>,
		mute: bool,
	) -> Self {
		let shared: Arc<RwLock<OscillatorShared<SAMPLE_RATE, N_CH>>> =
			Arc::new(RwLock::new(OscillatorShared {
				signal: frequencies_to_samples(SAMPLE_RATE.into(), &frequencies).multiply(),
				frequencies,
				mute,
			}));

		let stream_daemon = ResourceDaemon::new({
			let shared = shared.clone();
			let mut sample_idx = 0;

			move |quit_signal| {
				device
					.build_output_stream(
						&config.into(),
						move |output: &mut [f32], _| {
							let shared = shared.read().unwrap();
							if shared.mute {
								output.fill(0.);
							} else {
								debug_assert_eq!(output.len() % N_CH, 0);

								let signal = &shared.signal;

								let mut output_idx = 0;
								while output_idx < output.len() {
									let sample_idx_mod = sample_idx % signal.raw_buffer().len();
									let available = (output.len() - output_idx)
										.min(signal.raw_buffer().len() - sample_idx_mod);

									output[output_idx..output_idx + available].copy_from_slice(
										&signal.raw_buffer()
											[sample_idx_mod..sample_idx_mod + available],
									);
									output_idx += available;
									sample_idx += available;
								}
							}
						},
						move |err| {
							quit_signal.dispatch(AudioStreamError::SamplingError(err.to_string()));
						},
						None,
					)
					.map_err(|err| AudioStreamError::BuildFailed(err.to_string()))
					.and_then(|stream| {
						stream
							.play()
							.map(|()| stream)
							.map_err(|err| AudioStreamError::StartFailed(err.to_string()))
					})
			}
		});

		Self {
			shared,
			stream_daemon,
		}
	}

	#[must_use]
	pub fn state(&self) -> AudioStreamSamplingState {
		match self.stream_daemon.state() {
			resource_daemon::DaemonState::Holding => AudioStreamSamplingState::Sampling,
			resource_daemon::DaemonState::Quitting(reason)
			| resource_daemon::DaemonState::Quit(reason) => {
				AudioStreamSamplingState::Stopped(reason.unwrap_or(AudioStreamError::Cancelled))
			}
		}
	}

	/// # Panics
	/// - if the mutex guarding the internal state is poisoned.
	pub fn set_frequencies(&mut self, frequencies: &[f32]) {
		let mut shared = self.shared.write().unwrap();
		shared.frequencies = frequencies.to_vec();
		shared.signal = frequencies_to_samples(SAMPLE_RATE.into(), frequencies).multiply();
	}

	/// # Panics
	/// - if the mutex guarding the internal state is poisoned.
	#[must_use]
	pub fn frequencies(&self) -> Vec<f32> {
		self.shared.read().unwrap().frequencies.clone()
	}

	/// # Panics
	/// - if the mutex guarding the internal state is poisoned.
	pub fn set_mute(&mut self, mute: bool) {
		self.shared.write().unwrap().mute = mute;
	}

	/// # Panics
	/// - if the mutex guarding the internal state is poisoned.
	#[must_use]
	pub fn mute(&self) -> bool {
		self.shared.read().unwrap().mute
	}

	#[must_use]
	pub fn sample_rate(&self) -> usize {
		SAMPLE_RATE
	}

	#[must_use]
	pub fn n_of_channels(&self) -> usize {
		N_CH
	}
}

#[must_use]
pub fn frequencies_to_samples<const SAMPLE_RATE: usize>(
	samples: NOfSamples<SAMPLE_RATE>,
	frequencies: &[f32],
) -> InterleavedAudioBuffer<SAMPLE_RATE, 1, Vec<f32>> {
	let mut mono = (0..*samples)
		.map(move |i| {
			#[allow(clippy::cast_precision_loss)]
			frequencies
				.iter()
				.map(|f| f32::sin(TAU * f * (i as f32 / SAMPLE_RATE as f32)))
				.sum::<f32>()
		})
		.collect::<Vec<f32>>();

	let &abs_max = mono
		.iter()
		.max_by(|a, b| a.abs().total_cmp(&b.abs()))
		.unwrap_or(&1.);

	mono.iter_mut().for_each(|s| *s /= abs_max);

	InterleavedAudioBuffer::new(mono)
}

#[cfg(test)]
mod tests {
	use std::{thread::sleep, time::Duration};

	use super::*;

	#[test]
	fn test_440() {
		let oscillator = OscillatorBuilder::<44100, 1>::new(&[440.], false)
			.build()
			.unwrap();
		sleep(Duration::from_secs(10));
		assert!(!oscillator.mute());
	}
	#[test]
	fn test_440_333() {
		let _oscillator = OscillatorBuilder::<44100, 1>::new(&[440., 333.], false)
			.build()
			.unwrap();
		sleep(Duration::from_secs(10));
	}
}

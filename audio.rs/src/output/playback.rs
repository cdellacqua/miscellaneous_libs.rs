#![allow(clippy::cast_precision_loss)]

use std::{
	iter,
	sync::{Arc, Mutex},
};

use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Device, SampleFormat, SampleRate, Stream, SupportedStreamConfig,
};
use resource_daemon::ResourceDaemon;

use mutex_ext::LockExt;

use crate::{
	buffers::AudioFrame, AudioStreamBuilderError, AudioStreamError, AudioStreamSamplingState,
};

#[derive(Debug, Clone, Default)]
pub struct AudioPlayerBuilder<const N_CH: usize> {
	sample_rate: usize,
}

impl<const N_CH: usize> AudioPlayerBuilder<N_CH> {
	#[must_use]
	pub fn new(sample_rate: usize) -> Self {
		Self { sample_rate }
	}

	/// Build and start output stream
	///
	/// # Errors
	/// [`AudioStreamBuilderError`]
	///
	/// # Panics
	/// - if the output device default configuration doesn't use f32 as the sample format.
	pub fn build(&self) -> Result<AudioPlayer<N_CH>, AudioStreamBuilderError> {
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
			.try_with_sample_rate(SampleRate(self.sample_rate as u32))
			.ok_or(AudioStreamBuilderError::NoConfigFound)?;

		// TODO: normalize everything to f32 and accept any format?
		assert!(
			matches!(config.sample_format(), cpal::SampleFormat::F32),
			"expected F32 input stream"
		);

		Ok(AudioPlayer::new(device, config))
	}
}

pub type InterleavedSignalIter<const N_CH: usize> =
	Arc<Mutex<Box<dyn Iterator<Item = AudioFrame<N_CH, [f32; N_CH]>> + Send + Sync>>>;

pub struct AudioPlayer<const N_CH: usize> {
	sample_rate: usize,
	interleaved_signal: InterleavedSignalIter<N_CH>,
	stream_daemon: ResourceDaemon<Stream, AudioStreamError>,
}

impl<const N_CH: usize> AudioPlayer<N_CH> {
	fn new(device: Device, config: SupportedStreamConfig) -> Self {
		let interleaved_signal = Arc::new(Mutex::new(Box::new(iter::empty())
			as Box<dyn Iterator<Item = AudioFrame<N_CH, [f32; N_CH]>> + Send + Sync>));

		let sample_rate = config.sample_rate().0 as usize;

		let stream_daemon = ResourceDaemon::new({
			let interleaved_signal = interleaved_signal.clone();

			move |quit_signal| {
				device
					.build_output_stream(
						&config.into(),
						move |output: &mut [f32], _| {
							let output_frames = output.len() / N_CH;
							assert_eq!(output.len() % N_CH, 0);

							let frames = interleaved_signal
								.with_lock_mut(|m| m.take(output_frames).collect::<Vec<_>>());

							// clean the output as it may contain dirty values from a previous call
							output.fill(0.);

							frames.iter().zip(output.chunks_mut(N_CH)).for_each(
								|(samples, frame)| {
									frame.copy_from_slice(samples.as_slice());
								},
							);
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
			sample_rate,
			interleaved_signal,
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

	pub fn stop(&mut self) {
		self.stream_daemon.quit(AudioStreamError::Cancelled);
	}

	pub fn set_signal<
		Signal: Iterator<Item = AudioFrame<N_CH, [f32; N_CH]>> + Send + Sync + 'static,
	>(
		&mut self,
		signal: Signal,
	) {
		self.interleaved_signal.with_lock_mut(|f| {
			*f = Box::new(signal)
				as Box<dyn Iterator<Item = AudioFrame<N_CH, [f32; N_CH]>> + Send + Sync>;
		});
	}

	#[must_use]
	pub fn sample_rate(&self) -> usize {
		self.sample_rate
	}

	#[must_use]
	pub fn n_of_channels(&self) -> usize {
		N_CH
	}
}

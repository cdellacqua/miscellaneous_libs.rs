#![allow(clippy::cast_precision_loss)]

use std::{
	iter,
	sync::{Arc, Mutex},
};

use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Device, Stream, SupportedStreamConfig,
};
use resource_daemon::ResourceDaemon;

use mutex_ext::LockExt;

use crate::{
	buffers::AudioFrameTrait, AudioStreamBuilderError, AudioStreamError, AudioStreamSamplingState,
};

#[derive(Debug, Clone, Default)]
pub struct AudioPlayerBuilder {}

impl AudioPlayerBuilder {
	#[must_use]
	pub fn new() -> Self {
		Self {}
	}

	/// Build and start output stream
	///
	/// # Errors
	/// [`AudioStreamBuilderError`]
	///
	/// # Panics
	/// - if the output device default configuration doesn't use f32 as the sample format.
	pub fn build(&self) -> Result<AudioPlayer, AudioStreamBuilderError> {
		let device = cpal::default_host()
			.output_devices()
			.map_err(|_| AudioStreamBuilderError::UnableToListDevices)?
			.next()
			.ok_or(AudioStreamBuilderError::NoDeviceFound)?;

		let config = device
			.default_output_config()
			.map_err(|_| AudioStreamBuilderError::NoConfigFound)?;

		assert!(
			matches!(config.sample_format(), cpal::SampleFormat::F32),
			"expected F32 output stream"
		);

		Ok(AudioPlayer::new(device, config))
	}
}

type BoxedInterleavedSignalIter =
	Arc<Mutex<Box<dyn Iterator<Item = Box<dyn AudioFrameTrait>> + Send + Sync>>>;

pub struct AudioPlayer {
	sample_rate: usize,
	interleaved_signal: BoxedInterleavedSignalIter,
	n_of_channels: usize,
	stream_daemon: ResourceDaemon<Stream, AudioStreamError>,
}

impl AudioPlayer {
	fn new(device: Device, config: SupportedStreamConfig) -> Self {
		let interleaved_signal = Arc::new(Mutex::new(Box::new(iter::empty())
			as Box<dyn Iterator<Item = Box<dyn AudioFrameTrait>> + Send + Sync>));

		let n_of_channels = config.channels() as usize;
		let sample_rate = config.sample_rate().0 as usize;

		let stream_daemon = ResourceDaemon::new({
			let interleaved_signal = interleaved_signal.clone();

			move |quit_signal| {
				device
					.build_output_stream(
						&config.into(),
						move |output: &mut [f32], _| {
							let output_frames = output.len() / n_of_channels;
							assert_eq!(output.len() % n_of_channels, 0);

							let samples = interleaved_signal
								.with_lock_mut(|m| m.take(output_frames).collect::<Vec<_>>());

							// clean the output as it may contain dirty values from a previous call
							output.fill(0.);

							samples
								.iter()
								.zip(output.chunks_mut(n_of_channels))
								.for_each(|(sample, frame)| {
									frame.copy_from_slice(sample);
								});
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
			n_of_channels,
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

	pub fn set_signal<Signal: Iterator<Item = Box<dyn AudioFrameTrait>> + Send + Sync + 'static>(
		&mut self,
		signal: Signal,
	) {
		self.interleaved_signal.with_lock_mut(|f| {
			*f = Box::new(signal)
				as Box<dyn Iterator<Item = Box<dyn AudioFrameTrait>> + Send + Sync>;
		});
	}

	#[must_use]
	pub fn sample_rate(&self) -> usize {
		self.sample_rate
	}

	#[must_use]
	pub fn n_of_channels(&self) -> usize {
		self.n_of_channels
	}
}

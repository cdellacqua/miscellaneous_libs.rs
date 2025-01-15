#![allow(clippy::cast_precision_loss)]

use std::sync::{Arc, Mutex};

use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Device, Stream, SupportedStreamConfig,
};
use resource_daemon::ResourceDaemon;

use mutex_ext::LockExt;

use crate::{AudioOutputBuilderError, AudioOutputState, SamplingState};

#[derive(Debug, Clone, Default)]
pub struct AudioPlayerBuilder {}

impl AudioPlayerBuilder {
	#[must_use]
	pub fn new() -> Self {
		Self {}
	}

	///
	/// Build and start output stream
	///
	/// # Errors
	/// [`AudioOutputBuilderError`]
	///
	/// # Panics
	/// - if the output device default configuration doesn't use f32 as the sample format
	pub fn build(&self) -> Result<AudioPlayer, AudioOutputBuilderError> {
		let device = cpal::default_host()
			.output_devices()
			.map_err(|_| AudioOutputBuilderError::UnableToListDevices)?
			.next()
			.ok_or(AudioOutputBuilderError::NoDeviceFound)?;

		let config = device
			.default_output_config()
			.map_err(|_| AudioOutputBuilderError::NoConfigFound)?;

		assert!(
			matches!(config.sample_format(), cpal::SampleFormat::F32),
			"expected F32 output stream"
		);

		Ok(AudioPlayer::new(device, config))
	}
}

pub struct AudioPlayer {
	pub sample_rate: usize,
	mono_track: Arc<Mutex<Box<dyn Iterator<Item = f32> + Send>>>,
	pub n_of_channels: usize,
	stream_daemon: ResourceDaemon<Stream, AudioOutputState>,
}

struct NullTrack;
impl Iterator for NullTrack {
	type Item = f32;

	fn next(&mut self) -> Option<Self::Item> {
		Some(0.)
	}
}

impl AudioPlayer {
	fn new(device: Device, config: SupportedStreamConfig) -> Self {
		let mono_track = Arc::new(Mutex::new(
			Box::new(NullTrack) as Box<dyn Iterator<Item = f32> + Send>
		));

		let n_of_channels = config.channels() as usize;
		let sample_rate = config.sample_rate().0 as usize;

		let stream_daemon = ResourceDaemon::new({
			let mono_track = mono_track.clone();

			move |quit_signal| {
				device
					.build_output_stream(
						&config.into(),
						move |output: &mut [f32], _| {
							let output_frames = output.len() / n_of_channels;

							let samples = mono_track
								.with_lock_mut(|m| m.take(output_frames).collect::<Vec<_>>());

							// clean the output as it may contain dirty values from a previous call
							output.fill(0.);

							samples
								.iter()
								.zip(output.chunks_mut(n_of_channels))
								.for_each(|(&sample, frame)| {
									frame.fill(sample);
								});
						},
						move |err| {
							quit_signal.dispatch(AudioOutputState::SamplingError(err.to_string()));
						},
						None,
					)
					.map_err(|err| AudioOutputState::BuildFailed(err.to_string()))
					.and_then(|stream| {
						stream
							.play()
							.map(|()| stream)
							.map_err(|err| AudioOutputState::StartFailed(err.to_string()))
					})
			}
		});

		Self {
			sample_rate,
			mono_track,
			n_of_channels,
			stream_daemon,
		}
	}

	#[must_use]
	pub fn state(&self) -> SamplingState {
		match self.stream_daemon.state() {
			resource_daemon::DaemonState::Holding => SamplingState::Sampling,
			resource_daemon::DaemonState::Quitting(reason)
			| resource_daemon::DaemonState::Quit(reason) => {
				SamplingState::Stopped(reason.unwrap_or(AudioOutputState::Cancelled))
			}
		}
	}

	pub fn stop(&mut self) {
		self.stream_daemon.quit(AudioOutputState::Cancelled);
	}

	pub fn set_mono_track<Track: Iterator<Item = f32> + Send + 'static>(
		&mut self,
		mono_track: Track,
	) {
		self.mono_track
			.with_lock_mut(|f| *f = Box::new(mono_track) as Box<dyn Iterator<Item = f32> + Send>);
	}
}

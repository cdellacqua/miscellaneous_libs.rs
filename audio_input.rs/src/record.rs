use std::{
	sync::{Arc, Mutex},
	time::Duration,
};

use audio_analysis::buffers::InterleavedAudioSamples;
use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Device, Stream, SupportedStreamConfig,
};
use resource_daemon::ResourceDaemon;

use mutex_ext::LockExt;

use crate::common::{AudioInputBuilderError, AudioInputState, SamplingState};
// TODO: Record with start/collect/stop and capacity
pub struct AudioRecorderBuilder {
	capacity: Duration,
}

impl AudioRecorderBuilder {
	#[must_use]
	pub fn new(capacity: Duration) -> Self {
		Self { capacity }
	}

	/// Build and start recording the input stream
	///
	/// # Errors
	/// [`AudioInputBuilderError`]
	///
	/// # Panics
	/// - if the input device default configuration doesn't use f32 as the sample format
	pub fn build(&self) -> Result<AudioRecorder, AudioInputBuilderError> {
		let device = cpal::default_host()
			.input_devices()
			.map_err(|_| AudioInputBuilderError::UnableToListDevices)?
			.next()
			.ok_or(AudioInputBuilderError::NoDeviceFound)?;

		let config = device
			.default_input_config()
			.map_err(|_| AudioInputBuilderError::NoConfigFound)?;

		assert!(
			matches!(config.sample_format(), cpal::SampleFormat::F32),
			"expected F32 input stream"
		);

		Ok(AudioRecorder::new(self.capacity, device, config))
	}
}

pub struct AudioRecorder {
	sample_rate: usize,
	buffer: Arc<Mutex<Vec<f32>>>,
	capacity: Duration,
	n_of_channels: usize,
	stream_daemon: ResourceDaemon<Stream, AudioInputState>,
}

impl AudioRecorder {
	fn new(capacity: Duration, device: Device, config: SupportedStreamConfig) -> Self {
		let sample_rate = config.sample_rate().0 as usize;
		let n_of_channels = config.channels() as usize;

		let samples_per_channel = sample_rate * capacity.as_micros() as usize / 1_000_000;
		let buffer_size = n_of_channels * samples_per_channel;

		let buffer = Arc::new(Mutex::new({
			let mut buf = Vec::with_capacity(buffer_size);
			buf.fill(0.);
			buf
		}));

		let stream_daemon = ResourceDaemon::new({
			let buffer = buffer.clone();
			move |quit_signal| {
				device
					.build_input_stream(
						&config.into(),
						move |data, _| {
							buffer.with_lock_mut(|b| {
								for &v in data {
									b.push(v);
								}
							});
						},
						move |err| {
							quit_signal.dispatch(AudioInputState::SamplingError(err.to_string()));
						},
						None,
					)
					.map_err(|err| AudioInputState::BuildFailed(err.to_string()))
					.and_then(|stream| {
						stream
							.play()
							.map(|()| stream)
							.map_err(|err| AudioInputState::StartFailed(err.to_string()))
					})
			}
		});

		Self {
			sample_rate,
			buffer,
			capacity,
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
				SamplingState::Stopped(reason.unwrap_or(AudioInputState::Cancelled))
			}
		}
	}

	pub fn stop(&mut self) {
		self.stream_daemon.quit(AudioInputState::Cancelled);
	}

	/// Get the latest snapshot
	#[must_use]
	pub fn latest_snapshot(&self) -> InterleavedAudioSamples {
		InterleavedAudioSamples::new(self.n_of_channels, self.buffer.with_lock(Vec::clone))
	}

	#[must_use]
	pub fn capacity(&self) -> Duration {
		self.capacity
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

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
use ringbuffer::{AllocRingBuffer, RingBuffer};

use mutex_ext::LockExt;

pub struct InputStreamPollerBuilder {
	buffer_time_duration: Duration,
}

impl InputStreamPollerBuilder {
	#[must_use]
	pub fn new(buffer_time_duration: Duration) -> Self {
		Self {
			buffer_time_duration,
		}
	}

	///
	/// Build and start recording the input stream
	///
	/// # Errors
	/// [`InputStreamPollerBuilderError`]
	///
	/// # Panics
	/// - if the input device default configuration doesn't use f32 as the sample format
	pub fn build(&self) -> Result<InputStreamPoller, InputStreamPollerBuilderError> {
		let device = cpal::default_host()
			.input_devices()
			.map_err(|_| InputStreamPollerBuilderError::UnableToListDevices)?
			.next()
			.ok_or(InputStreamPollerBuilderError::NoDeviceFound)?;

		let config = device
			.default_input_config()
			.map_err(|_| InputStreamPollerBuilderError::NoConfigFound)?;

		assert!(
			matches!(config.sample_format(), cpal::SampleFormat::F32),
			"expected F32 input stream"
		);

		Ok(InputStreamPoller::new(
			self.buffer_time_duration,
			device,
			config,
		))
	}
}

#[derive(thiserror::Error, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum InputStreamPollerBuilderError {
	#[error("unable to list input devices")]
	UnableToListDevices,
	#[error("no available device found")]
	NoDeviceFound,
	#[error("no available stream configuration found")]
	NoConfigFound,
}

#[derive(thiserror::Error, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum InputStreamPollerState {
	#[error("unable to build stream")]
	BuildFailed(String),
	#[error("unable to start stream")]
	StartFailed(String),
	#[error("error while sampling")]
	SamplingError(String),
	#[error("stopped")]
	Cancelled,
}

pub struct InputStreamPoller {
	pub sample_rate: usize,
	ring_buffer: Arc<Mutex<AllocRingBuffer<f32>>>,
	pub n_of_channels: usize,
	stream_daemon: ResourceDaemon<Stream, InputStreamPollerState>,
}

#[derive(Debug, Clone)]
pub enum SamplingState {
	Sampling,
	Stopped(InputStreamPollerState),
}

impl InputStreamPoller {
	fn new(buffer_time_duration: Duration, device: Device, config: SupportedStreamConfig) -> Self {
		let sample_rate = config.sample_rate().0 as usize;
		let n_of_channels = config.channels() as usize;

		let samples_per_channel =
			sample_rate * buffer_time_duration.as_micros() as usize / 1_000_000;
		let buffer_size = n_of_channels * samples_per_channel;

		let ring_buffer = Arc::new(Mutex::new({
			let mut buf = AllocRingBuffer::new(buffer_size);
			buf.fill(0.);
			buf
		}));

		let stream_daemon = ResourceDaemon::new({
			let ring_buffer = ring_buffer.clone();
			move |quit_signal| {
				device
					.build_input_stream(
						&config.into(),
						move |data, _| {
							ring_buffer.with_lock_mut(|b| {
								for &v in data {
									b.push(v);
								}
							});
						},
						move |err| {
							quit_signal
								.dispatch(InputStreamPollerState::SamplingError(err.to_string()));
						},
						None,
					)
					.map_err(|err| InputStreamPollerState::BuildFailed(err.to_string()))
					.and_then(|stream| {
						stream
							.play()
							.map(|()| stream)
							.map_err(|err| InputStreamPollerState::StartFailed(err.to_string()))
					})
			}
		});

		Self {
			sample_rate,
			ring_buffer,
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
				SamplingState::Stopped(reason.unwrap_or(InputStreamPollerState::Cancelled))
			}
		}
	}

	pub fn stop(&mut self) {
		self.stream_daemon.quit(InputStreamPollerState::Cancelled);
	}

	///
	/// Get the latest frame snapshot
	///
	#[must_use]
	pub fn latest_snapshot(&self) -> InterleavedAudioSamples {
		InterleavedAudioSamples::new(
			self.ring_buffer.with_lock(RingBuffer::to_vec),
			self.n_of_channels,
		)
	}
}

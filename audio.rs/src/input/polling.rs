use std::{
	sync::{Arc, Mutex},
	time::Duration,
};

use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Device, Stream, SupportedStreamConfig,
};
use mutex_ext::LockExt;
use resource_daemon::ResourceDaemon;
use ringbuffer::{AllocRingBuffer, RingBuffer};

use crate::{
	buffers::{InterleavedAudioBufferFactory, InterleavedAudioBufferTraitMut},
	AudioStreamBuilderError, AudioStreamError, AudioStreamSamplingState,
};

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

	/// Build and start recording the input stream
	///
	/// # Errors
	/// [`AudioStreamBuilderError`]
	///
	/// # Panics
	/// - if the input device default configuration doesn't use f32 as the sample format.
	pub fn build(&self) -> Result<InputStreamPoller, AudioStreamBuilderError> {
		let device = cpal::default_host()
			.input_devices()
			.map_err(|_| AudioStreamBuilderError::UnableToListDevices)?
			.next()
			.ok_or(AudioStreamBuilderError::NoDeviceFound)?;

		let config = device
			.default_input_config()
			.map_err(|_| AudioStreamBuilderError::NoConfigFound)?;

		// TODO: normalize everything to f32 and accept any format?
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

pub struct InputStreamPoller {
	sample_rate: usize,
	ring_buffer: Arc<Mutex<AllocRingBuffer<f32>>>,
	n_of_channels: usize,
	stream_daemon: ResourceDaemon<Stream, AudioStreamError>,
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
			ring_buffer,
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

	/// Get the latest snapshot
	#[must_use]
	pub fn latest_snapshot(&self) -> Box<dyn InterleavedAudioBufferTraitMut> {
		InterleavedAudioBufferFactory::build_mut(
			self.n_of_channels,
			self.ring_buffer.with_lock(RingBuffer::to_vec),
		)
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

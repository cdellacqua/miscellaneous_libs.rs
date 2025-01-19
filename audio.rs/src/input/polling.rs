use std::{
	sync::{Arc, Mutex},
	time::Duration,
};

use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Device, SampleFormat, SampleRate, Stream, SupportedStreamConfig,
};
use mutex_ext::LockExt;
use resource_daemon::ResourceDaemon;
use ringbuffer::{AllocRingBuffer, RingBuffer};

use crate::{
	buffers::InterleavedAudioBuffer, AudioStreamBuilderError, AudioStreamError,
	AudioStreamSamplingState, DurationToNOfSamples,
};

pub struct InputStreamPollerBuilder<const N_CH: usize> {
	sample_rate: usize,
	buffer_time_duration: Duration,
}

impl<const N_CH: usize> InputStreamPollerBuilder<N_CH> {
	#[must_use]
	pub fn new(sample_rate: usize, buffer_time_duration: Duration) -> Self {
		Self {
			sample_rate,
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
	pub fn build(&self) -> Result<InputStreamPoller<N_CH>, AudioStreamBuilderError> {
		let device = cpal::default_host()
			.input_devices()
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

		Ok(InputStreamPoller::new(
			self.buffer_time_duration,
			device,
			config,
		))
	}
}

pub struct InputStreamPoller<const N_CH: usize> {
	sample_rate: usize,
	ring_buffer: Arc<Mutex<AllocRingBuffer<f32>>>,
	stream_daemon: ResourceDaemon<Stream, AudioStreamError>,
}

impl<const N_CH: usize> InputStreamPoller<N_CH> {
	fn new(buffer_time_duration: Duration, device: Device, config: SupportedStreamConfig) -> Self {
		let sample_rate = config.sample_rate().0 as usize;

		let samples_per_channel = buffer_time_duration.to_n_of_samples(sample_rate);
		let buffer_size = N_CH * samples_per_channel;

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
	pub fn latest_snapshot(&self) -> InterleavedAudioBuffer<N_CH, Vec<f32>> {
		InterleavedAudioBuffer::new(self.ring_buffer.with_lock(RingBuffer::to_vec))
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

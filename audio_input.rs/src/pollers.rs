use std::{
	sync::{Arc, Condvar, Mutex},
	thread::{spawn, JoinHandle},
	time::Duration,
};

use audio_analysis::buffers::InterleavedAudioSamples;
use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Device, SupportedStreamConfig,
};
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
	/// [`InputStreamPollerError`]
	///
	/// # Panics
	/// - if the input device default configuration doesn't use f32 as the sample format
	pub fn build(&self) -> Result<InputStreamPoller, InputStreamPollerError> {
		let device = cpal::default_host()
			.input_devices()
			.map_err(|_| InputStreamPollerError::UnableToListDevices)?
			.next()
			.ok_or(InputStreamPollerError::NoDeviceFound)?;

		let config = device
			.default_input_config()
			.map_err(|_| InputStreamPollerError::NoConfigFound)?;

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

#[derive(thiserror::Error, Debug)]
pub enum InputStreamPollerError {
	#[error("unable to list input devices")]
	UnableToListDevices,
	#[error("no available device found")]
	NoDeviceFound,
	#[error("no available stream configuration found")]
	NoConfigFound,
}

pub struct InputStreamPoller {
	pub sample_rate: usize,
	ring_buffer: Arc<Mutex<AllocRingBuffer<f32>>>,
	pub n_of_channels: usize,
	sampling_state: Arc<(Mutex<SamplingState>, Condvar)>,
	stream_supervisor: Option<JoinHandle<()>>,
}

#[derive(Debug, Clone)]
pub enum SamplingState {
	Sampling,
	Cancelled,
	Error(String),
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

		let sampling_state = Arc::new((Mutex::new(SamplingState::Sampling), Condvar::default()));

		let stream_supervisor = spawn({
			let ring_buffer = ring_buffer.clone();
			let sampling_state = sampling_state.clone();
			move || {
				let stream = device.build_input_stream(
					&config.into(),
					{
						move |data: &[f32], _: &_| {
							ring_buffer
								.with_lock_mut(|b| {
									for &v in data {
										b.push(v);
									}
								})
								.unwrap();
						}
					},
					{
						let sampling_state = sampling_state.clone();
						move |err| {
							let mut guard = sampling_state.0.lock().unwrap();
							if matches!(&*guard, SamplingState::Sampling) {
								*guard = SamplingState::Error(err.to_string());
							}
							sampling_state.1.notify_one();
						}
					},
					None,
				);
				match stream {
					Err(err) => {
						sampling_state
							.0
							.with_lock_mut(|sampling_error| {
								*sampling_error = SamplingState::Error(err.to_string());
							})
							.unwrap();
					}
					Ok(stream) => {
						if let Err(err) = stream.play() {
							sampling_state
								.0
								.with_lock_mut(|sampling_error| {
									*sampling_error = SamplingState::Error(err.to_string());
								})
								.unwrap();
						} else {
							let mut guard = sampling_state.0.lock().unwrap();
							while matches!(&*guard, SamplingState::Sampling) {
								guard = sampling_state.1.wait(guard).unwrap();
							}

							drop(stream);
						}
					}
				}
			}
		});

		Self {
			sample_rate,
			ring_buffer,
			n_of_channels,
			sampling_state,
			stream_supervisor: Some(stream_supervisor),
		}
	}

	///
	/// # Panics
	/// - if the mutex guarding the state is poisoned
	#[must_use]
	pub fn sampling_state(&self) -> SamplingState {
		self.sampling_state
			.0
			.with_lock(SamplingState::clone)
			.unwrap()
	}

	/// Get the latest frame snapshot
	///
	/// # Panics
	/// - if the mutex guarding the buffer is poisoned
	#[must_use]
	pub fn latest_snapshot(&self) -> InterleavedAudioSamples {
		InterleavedAudioSamples::new(
			self.ring_buffer
				.with_lock(ringbuffer::RingBuffer::to_vec)
				.unwrap(),
			self.n_of_channels,
		)
	}
}

impl Drop for InputStreamPoller {
	fn drop(&mut self) {
		{
			let mut guard = self.sampling_state.0.lock().unwrap();
			*guard = SamplingState::Cancelled;
			self.sampling_state.1.notify_one();
		}
		if let Some(supervisor) = self.stream_supervisor.take() {
			supervisor.join().unwrap();
		}
	}
}

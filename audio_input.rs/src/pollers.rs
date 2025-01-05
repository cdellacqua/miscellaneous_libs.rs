use std::{
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc, Mutex,
	},
	time::Duration,
};

use audio_analysis::buffers::InterleavedAudioSamples;
use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	BuildStreamError, Device, PlayStreamError, Stream, SupportedStreamConfig,
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
	/// # Errors
	/// [`InputStreamPollerError`]
	///
	/// # Panics
	/// - if the input device default configuration doesn't use f32 as the sample format
	pub fn start(&self) -> Result<InputStreamPoller, InputStreamPollerError> {
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

		InputStreamPoller::new(self.buffer_time_duration, &device, config)
	}
}

#[derive(thiserror::Error, Debug)]
pub enum InputStreamPollerError {
	#[error(transparent)]
	BuildStreamError(#[from] BuildStreamError),
	#[error(transparent)]
	PlayStreamError(#[from] PlayStreamError),
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
	sampling: Arc<AtomicBool>,
	_stream: Stream,
}

impl InputStreamPoller {
	fn new(
		buffer_time_duration: Duration,
		device: &Device,
		config: SupportedStreamConfig,
	) -> Result<Self, InputStreamPollerError> {
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

		let sampling = Arc::new(AtomicBool::new(true));

		let stream = device.build_input_stream(
			&config.into(),
			{
				let ring_buffer = ring_buffer.clone();
				move |data: &[f32], _: &_| {
					ring_buffer
						.with_lock(|b| {
							for &v in data {
								b.push(v);
							}
						})
						.unwrap();
				}
			},
			{
				let sampling = sampling.clone();
				move |err| {
					eprintln!("{err:?}");
					if sampling.load(Ordering::Relaxed) {
						sampling.store(false, Ordering::Relaxed);
					}
				}
			},
			None,
		)?;

		stream.play()?;

		Ok(Self {
			sample_rate,
			ring_buffer,
			n_of_channels,
			sampling,
			_stream: stream,
		})
	}

	#[must_use]
	pub fn sampling(&self) -> bool {
		self.sampling.load(Ordering::Relaxed)
	}

	/// Get the latest frame snapshot
	///
	/// # Panics
	/// - if the mutex guarding the buffer is poisoned
	#[must_use]
	pub fn latest_snapshot(&self) -> InterleavedAudioSamples {
		InterleavedAudioSamples::new(
			self.ring_buffer.with_lock(|b| b.to_vec()).unwrap(),
			self.n_of_channels,
		)
	}
}

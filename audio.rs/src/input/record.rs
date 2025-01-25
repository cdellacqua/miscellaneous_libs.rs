use std::{
	mem::replace,
	sync::{Arc, Mutex},
};

use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Device, SampleFormat, SampleRate, Stream, SupportedStreamConfig,
};
use resource_daemon::ResourceDaemon;

use mutex_ext::LockExt;

use crate::{
	buffers::InterleavedAudioBuffer,
	common::{AudioStreamBuilderError, AudioStreamError, AudioStreamSamplingState},
	NOfSamples,
};
pub struct AudioRecorderBuilder<const SAMPLE_RATE: usize, const N_CH: usize> {
	capacity: NOfSamples<SAMPLE_RATE>,
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> AudioRecorderBuilder<SAMPLE_RATE, N_CH> {
	#[must_use]
	pub const fn new(capacity: NOfSamples<SAMPLE_RATE>) -> Self {
		Self { capacity }
	}

	/// Build and start recording the input stream
	///
	/// # Errors
	/// [`AudioStreamBuilderError`]
	///
	/// # Panics
	/// - if the input device default configuration doesn't use f32 as the sample format.
	pub fn build(&self) -> Result<AudioRecorder<SAMPLE_RATE, N_CH>, AudioStreamBuilderError> {
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
			.try_with_sample_rate(SampleRate(SAMPLE_RATE as u32))
			.ok_or(AudioStreamBuilderError::NoConfigFound)?;

		// TODO: normalize everything to f32 and accept any format?
		assert!(
			matches!(config.sample_format(), cpal::SampleFormat::F32),
			"expected F32 input stream"
		);

		Ok(AudioRecorder::new(self.capacity, device, config))
	}
}

pub struct AudioRecorder<const SAMPLE_RATE: usize, const N_CH: usize> {
	buffer: Arc<Mutex<Vec<f32>>>,
	capacity: NOfSamples<SAMPLE_RATE>,
	capacity_bytes: usize,
	stream_daemon: ResourceDaemon<Stream, AudioStreamError>,
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> AudioRecorder<SAMPLE_RATE, N_CH> {
	fn new(
		capacity: NOfSamples<SAMPLE_RATE>,
		device: Device,
		config: SupportedStreamConfig,
	) -> Self {
		let buffer_size = N_CH * *capacity;

		let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::with_capacity(buffer_size)));

		let stream_daemon = ResourceDaemon::new({
			let buffer = buffer.clone();
			move |quit_signal| {
				device
					.build_input_stream(
						&config.into(),
						move |data, _| {
							buffer.with_lock_mut(|b| {
								data.iter()
									.take(buffer_size - b.len())
									.for_each(|&sample| b.push(sample));
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
			buffer,
			capacity,
			capacity_bytes: buffer_size,
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

	#[must_use]
	pub fn take(&mut self) -> InterleavedAudioBuffer<SAMPLE_RATE, N_CH, Vec<f32>> {
		InterleavedAudioBuffer::new(
			self.buffer
				.with_lock_mut(|b| replace(b, Vec::with_capacity(self.capacity_bytes))),
		)
	}

	/// Get the latest snapshot
	#[must_use]
	pub fn latest_snapshot(&self) -> InterleavedAudioBuffer<SAMPLE_RATE, N_CH, Vec<f32>> {
		InterleavedAudioBuffer::new(self.buffer.with_lock(Vec::clone))
	}

	#[must_use]
	pub fn capacity(&self) -> NOfSamples<SAMPLE_RATE> {
		self.capacity
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

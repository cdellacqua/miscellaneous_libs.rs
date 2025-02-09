use std::{
	mem::replace,
	sync::{Arc, Mutex},
};

use cpal::{
	traits::{DeviceTrait, StreamTrait},
	Device, Stream, SupportedStreamConfig,
};
use resource_daemon::ResourceDaemon;

use mutex_ext::LockExt;

use crate::{
	buffers::InterleavedAudioBuffer, common::{AudioStreamBuilderError, AudioStreamError, AudioStreamSamplingState}, device_provider, NOfSamples
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioRecorderBuilder<const SAMPLE_RATE: usize, const N_CH: usize> {
	capacity: NOfSamples<SAMPLE_RATE>,
	device_name: Option<String>,
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> AudioRecorderBuilder<SAMPLE_RATE, N_CH> {
	#[must_use]
	pub const fn new(capacity: NOfSamples<SAMPLE_RATE>, device_name: Option<String>) -> Self {
		Self {
			capacity,
			device_name,
		}
	}

	/// Build and start recording the input stream
	///
	/// # Errors
	/// [`AudioStreamBuilderError`]
	pub fn build(&self) -> Result<AudioRecorder<SAMPLE_RATE, N_CH>, AudioStreamBuilderError> {
		let (device, config) = device_provider(
			self.device_name.as_deref(),
			crate::IOMode::Input,
			N_CH,
			SAMPLE_RATE,
		)?;

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
								b.extend_from_slice(
									&data[0..data.len().min(buffer_size - data.len())],
								);
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
	pub fn snapshot(&self) -> InterleavedAudioBuffer<SAMPLE_RATE, N_CH, Vec<f32>> {
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

#[cfg(test)]
mod tests {
	use std::{thread::sleep, time::Duration};

	use crate::output::AudioPlayerBuilder;

	use super::*;

	#[test]
	#[ignore = "manually record and listen to the registered audio file"]
	fn test_manual() {
		let recorder = AudioRecorderBuilder::<44100, 1>::new(Duration::from_secs(2).into(), None)
			.build()
			.unwrap();
		sleep(recorder.capacity().into());
		let snapshot = recorder.snapshot();
		let mut player = AudioPlayerBuilder::<44100, 1>::new(None).build().unwrap();
		player.play(snapshot);
	}
}

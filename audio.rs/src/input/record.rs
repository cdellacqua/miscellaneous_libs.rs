use std::{
	mem::replace,
	sync::{Arc, Mutex},
};

use mutex_ext::LockExt;

use crate::{
	buffers::InterleavedAudioBuffer,
	common::{AudioStreamBuilderError, AudioStreamSamplingState},
	NOfFrames,
};

use super::{InputStream, InputStreamBuilder};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioRecorderBuilder<const SAMPLE_RATE: usize, const N_CH: usize> {
	capacity: NOfFrames<SAMPLE_RATE, N_CH>,
	device_name: Option<String>,
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> AudioRecorderBuilder<SAMPLE_RATE, N_CH> {
	#[must_use]
	pub const fn new(capacity: NOfFrames<SAMPLE_RATE, N_CH>, device_name: Option<String>) -> Self {
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
		let buffer_size = self.capacity.n_of_samples();
		let shared = Arc::new(Mutex::new(RecorderState {
			buffer_size,
			buffer: Vec::with_capacity(buffer_size),
		}));

		Ok(AudioRecorder::new(
			self.capacity,
			shared.clone(),
			InputStreamBuilder::new(
				self.device_name.clone(),
				Box::new(move |chunk| {
					shared.with_lock_mut(|shared| {
						shared.buffer.extend_from_slice(
							&chunk.raw_buffer()[0..chunk
								.raw_buffer()
								.len()
								.min(shared.buffer_size - chunk.raw_buffer().len())],
						);
					});
				}),
				None,
			)
			.build()?,
		))
	}
}

pub struct AudioRecorder<const SAMPLE_RATE: usize, const N_CH: usize> {
	capacity: NOfFrames<SAMPLE_RATE, N_CH>,
	shared: Arc<Mutex<RecorderState<SAMPLE_RATE, N_CH>>>,
	base_stream: InputStream<SAMPLE_RATE, N_CH>,
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> AudioRecorder<SAMPLE_RATE, N_CH> {
	fn new(
		capacity: NOfFrames<SAMPLE_RATE, N_CH>,
		shared: Arc<Mutex<RecorderState<SAMPLE_RATE, N_CH>>>,
		base_stream: InputStream<SAMPLE_RATE, N_CH>,
	) -> Self {
		Self {
			capacity,
			shared,
			base_stream,
		}
	}

	#[must_use]
	pub fn state(&self) -> AudioStreamSamplingState {
		self.base_stream.state()
	}

	#[must_use]
	pub fn take(&mut self) -> InterleavedAudioBuffer<SAMPLE_RATE, N_CH, Vec<f32>> {
		InterleavedAudioBuffer::new(self.shared.with_lock_mut(|shared| {
			replace(&mut shared.buffer, Vec::with_capacity(shared.buffer_size))
		}))
	}

	/// Get the latest snapshot
	#[must_use]
	pub fn snapshot(&self) -> InterleavedAudioBuffer<SAMPLE_RATE, N_CH, Vec<f32>> {
		InterleavedAudioBuffer::new(self.shared.with_lock(|shared| shared.buffer.clone()))
	}

	#[must_use]
	pub fn capacity(&self) -> NOfFrames<SAMPLE_RATE, N_CH> {
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

struct RecorderState<const SAMPLE_RATE: usize, const N_CH: usize> {
	buffer_size: usize,
	buffer: Vec<f32>,
}

#[cfg(test)]
mod tests {
	use std::{thread::sleep, time::Duration};

	use crate::output::AudioPlayerBuilder;

	use super::*;

	#[test]
	#[ignore = "manually record and listen to the registered audio file"]
	fn test_manual() {
		let mut recorder =
			AudioRecorderBuilder::<44100, 2>::new(Duration::from_secs(2).into(), None)
				.build()
				.unwrap();
		sleep(recorder.capacity().into());
		let snapshot = recorder.take();
		let mut player = AudioPlayerBuilder::<44100, 2>::new(None).build().unwrap();
		assert_eq!(player.state(), AudioStreamSamplingState::Sampling);
		player.play(snapshot);
	}
}

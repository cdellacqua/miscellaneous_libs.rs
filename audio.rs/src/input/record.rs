use std::{
	mem::replace,
	sync::{Arc, Mutex},
	time::Duration,
};

use mutex_ext::LockExt;

use crate::{
	buffers::InterleavedAudioBuffer,
	common::{AudioStreamBuilderError, AudioStreamSamplingState},
	NOfFrames, SampleRate, SamplingCtx,
};

use super::InputStream;

pub struct AudioRecorder {
	capacity: NOfFrames,
	shared: Arc<Mutex<RecorderState>>,
	base_stream: InputStream,
}

impl AudioRecorder {
	/// Build and start sampling an input stream
	///
	/// # Errors
	/// [`AudioStreamBuilderError`]
	pub fn new(
		sampling_ctx: SamplingCtx,
		capacity: NOfFrames,
		device_name: Option<&str>,
	) -> Result<Self, AudioStreamBuilderError> {
		let buffer_size = sampling_ctx.n_of_samples(capacity);
		let shared = Arc::new(Mutex::new(RecorderState {
			buffer_size,
			buffer: Vec::with_capacity(buffer_size),
		}));

		let base_stream = InputStream::new(
			sampling_ctx,
			device_name,
			Box::new({
				let shared = shared.clone();
				move |chunk| {
					shared.with_lock_mut(|shared| {
						shared.buffer.extend_from_slice(
							&chunk.raw_buffer()[0..chunk
								.raw_buffer()
								.len()
								.min(shared.buffer_size - chunk.raw_buffer().len())],
						);
					});
				}
			}),
			None,
		)?;

		Ok(Self {
			capacity,
			shared,
			base_stream,
		})
	}

	#[must_use]
	pub fn state(&self) -> AudioStreamSamplingState {
		self.base_stream.state()
	}

	#[must_use]
	pub fn take(&mut self) -> InterleavedAudioBuffer<Vec<f32>> {
		InterleavedAudioBuffer::new(
			self.sampling_ctx(),
			self.shared.with_lock_mut(|shared| {
				replace(&mut shared.buffer, Vec::with_capacity(shared.buffer_size))
			}),
		)
	}

	/// Get the latest snapshot
	#[must_use]
	pub fn snapshot(&self) -> InterleavedAudioBuffer<Vec<f32>> {
		InterleavedAudioBuffer::new(
			self.sampling_ctx(),
			self.shared.with_lock(|shared| shared.buffer.clone()),
		)
	}

	#[must_use]
	pub fn capacity(&self) -> NOfFrames {
		self.capacity
	}

	#[must_use]
	pub fn sampling_ctx(&self) -> SamplingCtx {
		self.base_stream.sampling_ctx()
	}

	#[must_use]
	pub fn sample_rate(&self) -> SampleRate {
		self.base_stream.sample_rate()
	}

	#[must_use]
	pub fn n_ch(&self) -> usize {
		self.base_stream.n_ch()
	}

	#[must_use]
	pub fn avg_input_delay(&self) -> Duration {
		self.base_stream.avg_input_delay()
	}
}

struct RecorderState {
	buffer_size: usize,
	buffer: Vec<f32>,
}

#[cfg(test)]
mod tests {
	use std::{thread::sleep, time::Duration};

	use crate::output::AudioPlayer;

	use super::*;

	#[test]
	#[ignore = "manually record and listen to the registered audio file"]
	fn test_manual() {
		let sampling_ctx = SamplingCtx::new(SampleRate(44100), 2);
		let mut recorder = AudioRecorder::new(
			sampling_ctx,
			sampling_ctx.to_n_of_frames(Duration::from_secs(2)),
			None,
		)
		.unwrap();
		sleep(sampling_ctx.to_duration(recorder.capacity()));
		let snapshot = recorder.take();
		let mut player = AudioPlayer::new(sampling_ctx, None).unwrap();
		assert_eq!(player.state(), AudioStreamSamplingState::Sampling);
		player.play(snapshot);
	}
}

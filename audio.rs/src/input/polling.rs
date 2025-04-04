use std::{
	sync::{Arc, Mutex},
	time::Duration,
};

use mutex_ext::LockExt;
use ringbuffer::{AllocRingBuffer, RingBuffer};

use crate::{
	buffers::InterleavedAudioBuffer, AudioStreamBuilderError, AudioStreamSamplingState, NOfFrames,
	SampleRate, SamplingCtx,
};

use super::InputStream;

pub struct InputStreamPoller {
	n_of_frames: NOfFrames,
	shared: Arc<Mutex<PollerState>>,
	base_stream: InputStream,
}

impl InputStreamPoller {
	/// Build and start sampling an input stream
	///
	/// # Errors
	/// [`AudioStreamBuilderError`]
	pub fn new(
		sampling_ctx: SamplingCtx,
		n_of_frames: NOfFrames,
		device_name: Option<&str>,
	) -> Result<Self, AudioStreamBuilderError> {
		let shared = Arc::new(Mutex::new({
			PollerState {
				buffer: {
					let mut buf = AllocRingBuffer::new(sampling_ctx.n_of_samples(n_of_frames));
					buf.fill(0.);
					buf
				},
				collected_frames: n_of_frames, // buffer pre-filled with 0.
			}
		}));

		let base_stream = InputStream::new(
			sampling_ctx,
			device_name,
			Box::new({
				let shared = shared.clone();
				move |chunk| {
					shared.with_lock_mut(|shared| {
						shared.buffer.extend_from_slice(chunk.raw_buffer());
						shared.collected_frames += chunk.n_of_frames();
					});
				}
			}),
			None,
		)?;

		Ok(Self {
			n_of_frames,
			shared,
			base_stream,
		})
	}

	#[must_use]
	pub fn state(&self) -> AudioStreamSamplingState {
		self.base_stream.state()
	}

	/// Get the latest snapshot of the internal buffer
	#[must_use]
	pub fn snapshot(&self) -> InterleavedAudioBuffer<Vec<f32>> {
		InterleavedAudioBuffer::new(
			self.sampling_ctx(),
			self.shared.with_lock(|shared| shared.buffer.to_vec()),
		)
	}

	/// Extract the last N frames from the internal buffer
	#[allow(clippy::missing_panics_doc)] // REASON: the code path when passing None always returns a Some(...)
	#[must_use]
	pub fn last_n_frames(&self, frames_to_extract: NOfFrames) -> InterleavedAudioBuffer<Vec<f32>> {
		self.frames_from_ref(frames_to_extract, None).unwrap().0
	}

	/// Extract N frames or less (depending on availability) from the
	/// internal buffer, starting from the specified
	/// `previously_collected_frames`. You can
	/// use this method to precisely concatenate signal snapshots
	/// together.
	///
	/// When passing `None` as `previously_collected_frames`, this
	/// method behaves like [`Self::last_n_frames`].
	///
	/// Note: if between the two snapshots the buffer has already been
	/// overwritten, None is returned.
	///
	/// # Panics
	/// - if the mutex guarding the internal data is poisoned.
	///
	/// Example (pseudocode):
	/// ```rust ignore
	/// let (beginning, collected_frames) = poller.frames_from_ref(NOfFrames::new(10), None);
	/// sleep(Duration::from_millis(100)); // assuming poller buffer is big enough to contain ~100ms of frames
	/// let (end, _) = poller.frames_from_ref(NOfFrames::new(10), Some(collected_frames));
	/// assert!(poller.snapshot().has_slice(beginning.concat(end)))
	/// ```
	#[must_use]
	pub fn frames_from_ref(
		&self,
		frames_to_extract: NOfFrames,
		previously_collected_frames: Option<NOfFrames>,
	) -> Option<(InterleavedAudioBuffer<Vec<f32>>, NOfFrames)> {
		let shared = self.shared.lock().unwrap();
		let collected_frames = shared.collected_frames;

		let skip = match previously_collected_frames {
			Some(prev) if collected_frames - prev >= self.n_of_frames => None,
			Some(prev) => Some(self.n_of_frames - (collected_frames - prev)),
			None => Some(self.n_of_frames - frames_to_extract.min(self.n_of_frames)),
		};

		skip.map(|skip| {
			(
				InterleavedAudioBuffer::new(self.sampling_ctx(), {
					let mut out =
						vec![0.; shared.buffer.len() - self.sampling_ctx().n_of_samples(skip)];
					if !out.is_empty() {
						shared
							.buffer
							.copy_to_slice(self.sampling_ctx().n_of_samples(skip), &mut out);
					}
					out
				}),
				collected_frames,
			)
		})
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
	pub fn n_of_frames(&self) -> NOfFrames {
		self.n_of_frames
	}

	#[must_use]
	pub fn avg_input_delay(&self) -> Duration {
		self.base_stream.avg_input_delay()
	}
}

struct PollerState {
	buffer: AllocRingBuffer<f32>,
	collected_frames: NOfFrames,
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
		let poller = InputStreamPoller::new(
			sampling_ctx,
			sampling_ctx.to_n_of_frames(Duration::from_secs(2)),
			None,
		)
		.unwrap();
		sleep(Duration::from_secs(2));
		let snapshot = poller.snapshot();
		let mut player = AudioPlayer::new(sampling_ctx, None).unwrap();
		player.play(snapshot);
	}
}

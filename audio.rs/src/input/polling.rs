use std::{
	sync::{Arc, Mutex},
	time::Duration,
};

use mutex_ext::LockExt;
use ringbuffer::{AllocRingBuffer, RingBuffer};

use crate::{
	buffers::InterleavedAudioBuffer,
	input::{InputStreamBuilder, StreamListener},
	AudioStreamBuilderError, AudioStreamSamplingState, NOfFrames,
};

use super::InputStream;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputStreamPollerBuilder<const SAMPLE_RATE: usize, const N_CH: usize> {
	n_of_frames: NOfFrames<SAMPLE_RATE, N_CH>,
	device_name: Option<String>,
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> InputStreamPollerBuilder<SAMPLE_RATE, N_CH> {
	#[must_use]
	pub const fn new(
		n_of_frames: NOfFrames<SAMPLE_RATE, N_CH>,
		device_name: Option<String>,
	) -> Self {
		Self {
			n_of_frames,
			device_name,
		}
	}

	/// Build and start recording the input stream
	///
	/// # Errors
	/// [`AudioStreamBuilderError`]
	pub fn build(&self) -> Result<InputStreamPoller<SAMPLE_RATE, N_CH>, AudioStreamBuilderError> {
		let shared = Arc::new(Mutex::new({
			PollerState {
				buffer: {
					let mut buf = AllocRingBuffer::new(self.n_of_frames.n_of_samples());
					buf.fill(0.);
					buf
				},
				collected_samples: self.n_of_frames, // buffer pre-filled with 0.
			}
		}));

		Ok(InputStreamPoller::new(
			self.n_of_frames,
			shared.clone(),
			InputStreamBuilder::new(
				self.device_name.clone(),
				Box::new(InputStreamPollerListener::<SAMPLE_RATE, N_CH>::new(shared)),
			)
			.build()?,
		))
	}
}

pub struct InputStreamPoller<const SAMPLE_RATE: usize, const N_CH: usize> {
	n_of_samples: NOfFrames<SAMPLE_RATE, N_CH>,
	shared: Arc<Mutex<PollerState<SAMPLE_RATE, N_CH>>>,
	base_stream: InputStream<SAMPLE_RATE, N_CH>,
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> InputStreamPoller<SAMPLE_RATE, N_CH> {
	fn new(
		n_of_samples: NOfFrames<SAMPLE_RATE, N_CH>,
		shared: Arc<Mutex<PollerState<SAMPLE_RATE, N_CH>>>,
		base_stream: InputStream<SAMPLE_RATE, N_CH>,
	) -> Self {
		Self {
			n_of_samples,
			shared,
			base_stream,
		}
	}

	#[must_use]
	pub fn state(&self) -> AudioStreamSamplingState {
		self.base_stream.state()
	}

	/// Get the latest snapshot of the internal buffer
	#[must_use]
	pub fn snapshot(&self) -> InterleavedAudioBuffer<SAMPLE_RATE, N_CH, Vec<f32>> {
		InterleavedAudioBuffer::new(self.shared.with_lock(|shared| shared.buffer.to_vec()))
	}

	/// Extract the last N samples from the internal buffer
	#[allow(clippy::missing_panics_doc)] // REASON: the code path when passing None always returns a Some(...)
	#[must_use]
	pub fn last_n_samples(
		&self,
		samples_to_extract: NOfFrames<SAMPLE_RATE, N_CH>,
	) -> InterleavedAudioBuffer<SAMPLE_RATE, N_CH, Vec<f32>> {
		self.samples_from_ref(samples_to_extract, None).unwrap().0
	}

	/// Extract N samples or less (depending on availability) from the
	/// internal buffer, starting from the specified
	/// `previously_collected_samples`. You can
	/// use this method to precisely concatenate signal snapshots
	/// together.
	///
	/// When passing `None` as `previously_collected_samples`, this
	/// method behaves like [`Self::last_n_samples`].
	///
	/// Note: if between the two snapshots the buffer has already been
	/// overwritten, None is returned.
	///
	/// # Panics
	/// - if the mutex guarding the internal data is poisoned.
	///
	/// Example (pseudocode):
	/// ```rust ignore
	/// let (beginning, collected_samples) = poller.samples_from_ref(NOfFrames::new(10), None);
	/// sleep(Duration::from_millis(100)); // assuming poller buffer is big enough to contain ~100ms of samples
	/// let (end, _) = poller.samples_from_ref(NOfFrames::new(10), Some(collected_samples));
	/// assert!(poller.snapshot().has_slice(beginning.concat(end)))
	/// ```
	#[must_use]
	pub fn samples_from_ref(
		&self,
		samples_to_extract: NOfFrames<SAMPLE_RATE, N_CH>,
		previously_collected_samples: Option<NOfFrames<SAMPLE_RATE, N_CH>>,
	) -> Option<(
		InterleavedAudioBuffer<SAMPLE_RATE, N_CH, Vec<f32>>,
		NOfFrames<SAMPLE_RATE, N_CH>,
	)> {
		let shared = self.shared.lock().unwrap();
		let collected_samples = shared.collected_samples;

		let skip = match previously_collected_samples {
			Some(prev) if collected_samples - prev >= self.n_of_samples => None,
			Some(prev) => Some(self.n_of_samples - (collected_samples - prev)),
			None => Some(self.n_of_samples - samples_to_extract.min(self.n_of_samples)),
		};

		skip.map(|skip| {
			(
				InterleavedAudioBuffer::new({
					let mut out = vec![0.; shared.buffer.len() - skip.inner()];
					if !out.is_empty() {
						shared.buffer.copy_to_slice(skip.inner(), &mut out);
					}
					out
				}),
				collected_samples,
			)
		})
	}

	/// Number of sampling points, regardless of the number of channels.
	#[must_use]
	pub fn n_of_samples(&self) -> NOfFrames<SAMPLE_RATE, N_CH> {
		self.n_of_samples
	}

	#[must_use]
	pub fn sample_rate(&self) -> usize {
		SAMPLE_RATE
	}

	#[must_use]
	pub fn n_of_channels(&self) -> usize {
		N_CH
	}

	#[must_use]
	pub fn avg_input_delay(&self) -> Duration {
		todo!()
	}
}

struct PollerState<const SAMPLE_RATE: usize, const N_CH: usize> {
	buffer: AllocRingBuffer<f32>,
	collected_samples: NOfFrames<SAMPLE_RATE, N_CH>,
}

struct InputStreamPollerListener<const SAMPLE_RATE: usize, const N_CH: usize> {
	shared: Arc<Mutex<PollerState<SAMPLE_RATE, N_CH>>>,
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> InputStreamPollerListener<SAMPLE_RATE, N_CH> {
	fn new(shared: Arc<Mutex<PollerState<SAMPLE_RATE, N_CH>>>) -> Self {
		Self { shared }
	}
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> StreamListener<SAMPLE_RATE, N_CH>
	for InputStreamPollerListener<SAMPLE_RATE, N_CH>
{
	fn on_data(&mut self, chunk: InterleavedAudioBuffer<SAMPLE_RATE, N_CH, &[f32]>) {
		self.shared.with_lock_mut(|shared| {
			shared.buffer.extend_from_slice(chunk.raw_buffer());
			shared.collected_samples += chunk.n_of_frames();
		});
	}

	fn on_error(&mut self, _reason: &str) {
		// ignored, it will just stop sampling
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
		let poller = InputStreamPollerBuilder::<44100, 2>::new(Duration::from_secs(2).into(), None)
			.build()
			.unwrap();
		sleep(poller.n_of_samples().into());
		let snapshot = poller.snapshot();
		let mut player = AudioPlayerBuilder::<44100, 2>::new(None).build().unwrap();
		player.play(snapshot);
	}
}

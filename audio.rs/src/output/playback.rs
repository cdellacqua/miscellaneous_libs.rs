#![allow(clippy::cast_precision_loss)]

use std::{thread::sleep, time::Duration};

use mutex_ext::{CondvarExt, LockExt, ReactiveCondvar};

use crate::{
	buffers::InterleavedAudioBuffer, AudioStreamBuilderError, AudioStreamSamplingState, NOfFrames,
	SampleRate, SamplingCtx,
};

use super::OutputStream;

pub struct AudioPlayer {
	shared: ReactiveCondvar<PlayerState>,
	base_stream: OutputStream,
}

impl AudioPlayer {
	/// Build and start sampling an input stream
	///
	/// # Errors
	/// [`AudioStreamBuilderError`]
	pub fn new(
		sampling_ctx: SamplingCtx,
		device_name: Option<&str>,
	) -> Result<Self, AudioStreamBuilderError> {
		let shared = ReactiveCondvar::new(PlayerState {
			frame_idx: NOfFrames(0),
			signal: InterleavedAudioBuffer::new(sampling_ctx, vec![]),
			end_of_signal: true,
		});

		let base_stream = OutputStream::new(
			sampling_ctx,
			device_name,
			Box::new({
				let shared = shared.clone();
				move |mut chunk| {
					let output_frames = chunk.n_of_frames();
					let should_notify = shared.mutex().with_lock_mut(|shared| {
						if shared.end_of_signal {
							chunk.raw_buffer_mut().fill(0.);
							false
						} else {
							let clamped_frames =
								output_frames.min(shared.signal.n_of_frames() - shared.frame_idx);

							chunk.raw_buffer_mut()
								[..sampling_ctx.frames_to_samples(clamped_frames)]
								.copy_from_slice(
									&shared.signal.raw_buffer()[sampling_ctx
										.frames_to_samples(shared.frame_idx)
										..sampling_ctx
											.frames_to_samples(shared.frame_idx + clamped_frames)],
								);
							chunk.raw_buffer_mut()
								[sampling_ctx.frames_to_samples(clamped_frames)..]
								.fill(0.);

							shared.frame_idx += clamped_frames;

							if shared.frame_idx == shared.signal.n_of_frames() {
								shared.end_of_signal = true;
								true
							} else {
								false
							}
						}
					});
					if should_notify {
						shared.condvar().notify_all();
					}
				}
			}),
			None,
		)?;

		Ok(Self {
			shared,
			base_stream,
		})
	}

	#[must_use]
	pub fn state(&self) -> AudioStreamSamplingState {
		self.base_stream.state()
	}

	/// Note: the wait time is based on when the iterator is exhausted and an estimate on when the output
	/// device should play the last samples.
	/// # Panics
	/// - if the mutex guarding the state of the associated thread is poisoned
	pub fn wait(&self) {
		self.shared.wait_while(|p| !p.end_of_signal);
		sleep(self.base_stream.avg_output_delay());
	}

	/// # Panics
	/// - if the mutex guarding the internal state is poisoned.
	pub fn set_signal(&mut self, signal: InterleavedAudioBuffer<Vec<f32>>) {
		self.shared.with_lock_mut(|shared| {
			shared.signal = signal;
			shared.frame_idx = NOfFrames(0);
			shared.end_of_signal = false;
		});
	}

	/// Note: blocking, `set_signal` is the non-blocking equivalent.
	///
	/// Note: the wait time is based on when the iterator is exhausted and an estimate on when the output
	/// device should play the last samples.
	pub fn play(&mut self, signal: InterleavedAudioBuffer<Vec<f32>>) {
		self.set_signal(signal);
		self.wait();
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
	pub fn avg_output_delay(&self) -> Duration {
		self.base_stream.avg_output_delay()
	}
}

struct PlayerState {
	signal: InterleavedAudioBuffer<Vec<f32>>,
	end_of_signal: bool,
	frame_idx: NOfFrames,
}

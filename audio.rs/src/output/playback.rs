#![allow(clippy::cast_precision_loss)]

use std::{
	sync::{Arc, Condvar, Mutex},
	thread::sleep,
	time::Duration,
};

use mutex_ext::LockExt;

use crate::{
	buffers::InterleavedAudioBuffer, AudioStreamBuilderError, AudioStreamSamplingState, NOfFrames,
};

use super::{OutputStream, OutputStreamBuilder};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AudioPlayerBuilder<const SAMPLE_RATE: usize, const N_CH: usize> {
	device_name: Option<String>,
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> AudioPlayerBuilder<SAMPLE_RATE, N_CH> {
	#[must_use]
	pub const fn new(device_name: Option<String>) -> Self {
		Self { device_name }
	}

	/// Build and start output stream
	///
	/// # Errors
	/// [`AudioStreamBuilderError`]
	pub fn build(&self) -> Result<AudioPlayer<SAMPLE_RATE, N_CH>, AudioStreamBuilderError> {
		let shared = Arc::new((
			Mutex::new(PlayerState {
				frame_idx: NOfFrames::new(0),
				signal: InterleavedAudioBuffer::new(vec![]),
				end_of_signal: true,
			}),
			Condvar::default(),
		));

		Ok(AudioPlayer::new(
			shared.clone(),
			OutputStreamBuilder::new(
				self.device_name.clone(),
				Box::new(move |mut chunk| {
					let output_frames = chunk.n_of_frames();
					let should_notify = shared.0.with_lock_mut(|shared| {
						if shared.end_of_signal {
							chunk.raw_buffer_mut().fill(0.);
							false
						} else {
							let clamped_frames =
								output_frames.min(shared.signal.n_of_frames() - shared.frame_idx);

							chunk.raw_buffer_mut()[..clamped_frames.n_of_samples()]
								.copy_from_slice(
									&shared.signal.raw_buffer()[shared.frame_idx.n_of_samples()
										..(shared.frame_idx + clamped_frames).n_of_samples()],
								);
							chunk.raw_buffer_mut()[clamped_frames.n_of_samples()..].fill(0.);

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
						shared.1.notify_all();
					}
				}),
				None,
			)
			.build()?,
		))
	}
}

pub struct AudioPlayer<const SAMPLE_RATE: usize, const N_CH: usize> {
	shared: Arc<(Mutex<PlayerState<SAMPLE_RATE, N_CH>>, Condvar)>,
	base_stream: OutputStream<SAMPLE_RATE, N_CH>,
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> AudioPlayer<SAMPLE_RATE, N_CH> {
	fn new(
		shared: Arc<(Mutex<PlayerState<SAMPLE_RATE, N_CH>>, Condvar)>,
		base_stream: OutputStream<SAMPLE_RATE, N_CH>,
	) -> Self {
		Self {
			shared,
			base_stream,
		}
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
		let guard = self
			.shared
			.1
			.wait_while(self.shared.0.lock().unwrap(), |p| !p.end_of_signal)
			.unwrap();
		drop(guard);
		sleep(self.base_stream.avg_output_delay());
	}

	/// # Panics
	/// - if the mutex guarding the internal state is poisoned.
	pub fn set_signal(&mut self, signal: InterleavedAudioBuffer<SAMPLE_RATE, N_CH, Vec<f32>>) {
		self.shared.0.with_lock_mut(|shared| {
			shared.signal = signal;
			shared.frame_idx = NOfFrames::new(0);
			shared.end_of_signal = false;
		});
	}

	/// Note: blocking, `set_signal` is the non-blocking equivalent.
	///
	/// Note: the wait time is based on when the iterator is exhausted and an estimate on when the output
	/// device should play the last samples.
	pub fn play(&mut self, signal: InterleavedAudioBuffer<SAMPLE_RATE, N_CH, Vec<f32>>) {
		self.set_signal(signal);
		self.wait();
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
	pub fn avg_output_delay(&self) -> Duration {
		self.base_stream.avg_output_delay()
	}
}

struct PlayerState<const SAMPLE_RATE: usize, const N_CH: usize> {
	signal: InterleavedAudioBuffer<SAMPLE_RATE, N_CH, Vec<f32>>,
	end_of_signal: bool,
	frame_idx: NOfFrames<SAMPLE_RATE, N_CH>,
}

#![allow(clippy::cast_precision_loss)]

use std::{
	iter,
	sync::{Arc, Condvar, Mutex},
	thread::sleep,
	time::Duration,
};

use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Device, SampleFormat, SampleRate, Stream, SupportedStreamConfig,
};
use resource_daemon::ResourceDaemon;

use mutex_ext::LockExt;

use crate::{
	buffers::AudioFrame, AudioStreamBuilderError, AudioStreamError, AudioStreamSamplingState,
};

#[derive(Debug, Clone, Default)]
pub struct AudioPlayerBuilder<const SAMPLE_RATE: usize, const N_CH: usize> {}

impl<const SAMPLE_RATE: usize, const N_CH: usize> AudioPlayerBuilder<SAMPLE_RATE, N_CH> {
	#[must_use]
	pub const fn new() -> Self {
		Self {}
	}

	/// Build and start output stream
	///
	/// # Errors
	/// [`AudioStreamBuilderError`]
	///
	/// # Panics
	/// - if the output device default configuration doesn't use f32 as the sample format.
	pub fn build(&self) -> Result<AudioPlayer<SAMPLE_RATE, N_CH>, AudioStreamBuilderError> {
		let device = cpal::default_host()
			.output_devices()
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

		Ok(AudioPlayer::new(device, config))
	}
}

type InterleavedSignalIter<const SAMPLE_RATE: usize, const N_CH: usize> =
	Arc<Mutex<Box<dyn Iterator<Item = AudioFrame<N_CH, [f32; N_CH]>> + Send + Sync>>>;

#[derive(Debug, Clone, Copy, Default)]
struct PlayingState {
	is_playing: bool,
	last_frame_delay: Option<Duration>,
}

pub struct AudioPlayer<const SAMPLE_RATE: usize, const N_CH: usize> {
	interleaved_signal: InterleavedSignalIter<SAMPLE_RATE, N_CH>,
	stream_daemon: ResourceDaemon<Stream, AudioStreamError>,
	playing: Arc<(Mutex<PlayingState>, Condvar)>,
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> AudioPlayer<SAMPLE_RATE, N_CH> {
	fn new(device: Device, config: SupportedStreamConfig) -> Self {
		let interleaved_signal = Arc::new(Mutex::new(Box::new(iter::empty())
			as Box<dyn Iterator<Item = AudioFrame<N_CH, [f32; N_CH]>> + Send + Sync>));

		let playing = Arc::new((Mutex::new(PlayingState::default()), Condvar::default()));

		let stream_daemon = ResourceDaemon::new({
			let interleaved_signal = interleaved_signal.clone();
			let playing = playing.clone();

			move |quit_signal| {
				device
					.build_output_stream(
						&config.into(),
						move |output: &mut [f32], info| {
							let output_frames = output.len() / N_CH;
							assert_eq!(output.len() % N_CH, 0);

							let frames = interleaved_signal
								.with_lock_mut(|m| m.take(output_frames).collect::<Vec<_>>());

							if frames.is_empty() {
								let mut guard = playing.0.lock().unwrap();
								if guard.is_playing {
									*guard = PlayingState {
										is_playing: false,
										last_frame_delay: info
											.timestamp()
											.playback
											.duration_since(&info.timestamp().callback),
									};
									playing.1.notify_all();
								}
							}

							// clean the output as it may contain dirty values from a previous call
							output.fill(0.);

							frames
								.iter()
								.zip(output.chunks_mut(N_CH))
								.for_each(|(src, dst)| {
									dst.copy_from_slice(src.samples());
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
			interleaved_signal,
			stream_daemon,
			playing,
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

	/// Note: the wait time is based on when the iterator is exhausted and an estimate on when the output
	/// device should play the last samples.
	/// # Panics
	/// - if the mutex guarding the state of the associated thread is poisoned
	pub fn wait(&self) {
		let guard = self
			.playing
			.1
			.wait_while(self.playing.0.lock().unwrap(), |p| p.is_playing)
			.unwrap();
		let last_frame_delay = guard.last_frame_delay;
		drop(guard);
		if let Some(last_frame_delay) = last_frame_delay {
			sleep(last_frame_delay);
		}
	}

	pub fn stop(&mut self) {
		self.stream_daemon.quit(AudioStreamError::Cancelled);
	}

	pub fn set_signal<
		Signal: Iterator<Item = AudioFrame<N_CH, [f32; N_CH]>> + Send + Sync + 'static,
	>(
		&mut self,
		signal: Signal,
	) {
		self.playing.0.with_lock_mut(|p| {
			*p = PlayingState {
				is_playing: true,
				last_frame_delay: None,
			}
		});
		self.interleaved_signal.with_lock_mut(|f| {
			*f = Box::new(signal)
				as Box<dyn Iterator<Item = AudioFrame<N_CH, [f32; N_CH]>> + Send + Sync>;
		});
	}

	/// Note: blocking, `set_signal` is the non-blocking equivalent.
	/// Note: the wait time is based on when the iterator is exhausted and an estimate on when the output
	/// device should play the last samples.
	pub fn play<Signal: Iterator<Item = AudioFrame<N_CH, [f32; N_CH]>> + Send + Sync + 'static>(
		&mut self,
		signal: Signal,
	) {
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
}

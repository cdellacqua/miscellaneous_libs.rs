#![allow(clippy::cast_precision_loss)]

use std::{
	sync::{
		mpsc::{sync_channel, SyncSender, TryRecvError},
		Arc, Condvar, Mutex,
	},
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
	buffers::InterleavedAudioBuffer, AudioStreamBuilderError, AudioStreamError,
	AudioStreamSamplingState, NOfSamples,
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

#[derive(Debug, Clone, Copy, Default)]
struct PlayingState {
	is_playing: bool,
	last_frame_delay: Option<Duration>,
}

pub struct AudioPlayer<const SAMPLE_RATE: usize, const N_CH: usize> {
	signal_tx: SyncSender<InterleavedAudioBuffer<SAMPLE_RATE, N_CH, Vec<f32>>>,
	stream_daemon: ResourceDaemon<Stream, AudioStreamError>,
	playing: Arc<(Mutex<PlayingState>, Condvar)>,
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> AudioPlayer<SAMPLE_RATE, N_CH> {
	fn new(device: Device, config: SupportedStreamConfig) -> Self {
		let (signal_tx, signal_rx) =
			sync_channel::<InterleavedAudioBuffer<SAMPLE_RATE, N_CH, Vec<f32>>>(0);

		let playing = Arc::new((Mutex::new(PlayingState::default()), Condvar::default()));

		let stream_daemon = ResourceDaemon::new({
			let playing = playing.clone();

			let mut currently_playing = None;

			move |quit_signal| {
				device
					.build_output_stream(
						&config.into(),
						move |output: &mut [f32], info| match &mut currently_playing {
							None => match signal_rx.try_recv() {
								Err(TryRecvError::Disconnected) => {
									panic!("internal error: broken channel")
								}
								Err(TryRecvError::Empty) => (),
								Ok(new_signal) => {
									currently_playing = Some((NOfSamples::new(0), new_signal));
								}
							},
							Some((frame_idx, signal)) => {
								if *frame_idx >= signal.n_of_samples() {
									output.fill(0.);
									
									currently_playing = None;
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
								} else {
									let output_frames = NOfSamples::new(output.len() / N_CH);
									debug_assert_eq!(output.len() % N_CH, 0);

									let clamped_frames =
										output_frames.min(signal.n_of_samples() - *frame_idx);

									output[..*clamped_frames].copy_from_slice(
										&signal.raw_buffer()[**frame_idx * N_CH
											..(**frame_idx + *clamped_frames) * N_CH],
									);
									output[*clamped_frames..].fill(0.);

									*frame_idx += clamped_frames;
								}
							}
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
			signal_tx,
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

	/// # Panics
	/// - if the mutex guarding the internal state is poisoned.
	pub fn set_signal(&mut self, signal: InterleavedAudioBuffer<SAMPLE_RATE, N_CH, Vec<f32>>) {
		self.playing.0.with_lock_mut(|p| {
			*p = PlayingState {
				is_playing: true,
				last_frame_delay: None,
			}
		});
		self.signal_tx
			.send(signal)
			.expect("internal error: broken channel");
	}

	/// Note: blocking, `set_signal` is the non-blocking equivalent.
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
}

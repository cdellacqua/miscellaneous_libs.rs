use std::{
	sync::{Arc, Mutex},
	time::Duration,
};

use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Device, SampleFormat, SampleRate, Stream, SupportedStreamConfig,
};
use math_utils::moving_avg::MovingAverage;
use mutex_ext::LockExt;
use resource_daemon::ResourceDaemon;
use ringbuffer::{AllocRingBuffer, RingBuffer};

use crate::{
	buffers::InterleavedAudioBuffer, AudioStreamBuilderError, AudioStreamError,
	AudioStreamSamplingState, NOfSamples,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputStreamPollerBuilder<const SAMPLE_RATE: usize, const N_CH: usize> {
	n_of_samples: NOfSamples<SAMPLE_RATE>,
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> InputStreamPollerBuilder<SAMPLE_RATE, N_CH> {
	#[must_use]
	pub const fn new(n_of_samples: NOfSamples<SAMPLE_RATE>) -> Self {
		Self { n_of_samples }
	}

	/// Build and start recording the input stream
	///
	/// # Errors
	/// [`AudioStreamBuilderError`]
	///
	/// # Panics
	/// - if the input device default configuration doesn't use f32 as the sample format.
	pub fn build(&self) -> Result<InputStreamPoller<SAMPLE_RATE, N_CH>, AudioStreamBuilderError> {
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

		Ok(InputStreamPoller::new(self.n_of_samples, device, config))
	}
}

struct PollerState<const SAMPLE_RATE: usize> {
	buffer: AllocRingBuffer<f32>,
	collected_samples: NOfSamples<SAMPLE_RATE>,
	input_delay_moving_avg: MovingAverage<Duration>,
}

pub struct InputStreamPoller<const SAMPLE_RATE: usize, const N_CH: usize> {
	shared: Arc<Mutex<PollerState<SAMPLE_RATE>>>,
	stream_daemon: ResourceDaemon<Stream, AudioStreamError>,
	n_of_samples: NOfSamples<SAMPLE_RATE>,
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> InputStreamPoller<SAMPLE_RATE, N_CH> {
	fn new(
		n_of_samples: NOfSamples<SAMPLE_RATE>,
		device: Device,
		config: SupportedStreamConfig,
	) -> Self {
		let shared = Arc::new(Mutex::new({
			PollerState {
				buffer: {
					let mut buf = AllocRingBuffer::new(N_CH * *n_of_samples);
					buf.fill(0.);
					buf
				},
				collected_samples: n_of_samples, // buffer pre-filled with 0.
				input_delay_moving_avg: MovingAverage::new(10),
			}
		}));

		let stream_daemon = ResourceDaemon::new({
			let shared = shared.clone();
			move |quit_signal| {
				device
					.build_input_stream(
						&config.into(),
						move |data, info| {
							let output_buffer_frames =
								NOfSamples::<SAMPLE_RATE>::new(data.len() / N_CH);

							shared.with_lock_mut(
								|PollerState {
								     buffer,
								     ref mut collected_samples,
								     ref mut input_delay_moving_avg,
								 }| {
									buffer.extend_from_slice(data);

									// assert_eq!(data.len() % N_CH, 0);
									*collected_samples += output_buffer_frames;

									input_delay_moving_avg.push(
										info.timestamp()
											.callback
											.duration_since(&info.timestamp().capture)
											.unwrap_or(Duration::ZERO) + output_buffer_frames
											.to_duration(),
									);
								},
							);
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
			shared,
			stream_daemon,
			n_of_samples,
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
		samples_to_extract: NOfSamples<SAMPLE_RATE>,
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
	/// let (beginning, collected_samples) = poller.samples_from_ref(NOfSamples::new(10), None);
	/// sleep(Duration::from_millis(100)); // assuming poller buffer is big enough to contain ~100ms of samples
	/// let (end, _) = poller.samples_from_ref(NOfSamples::new(10), Some(collected_samples));
	/// assert!(poller.snapshot().has_slice(beginning.concat(end)))
	/// ```
	#[must_use]
	pub fn samples_from_ref(
		&self,
		samples_to_extract: NOfSamples<SAMPLE_RATE>,
		previously_collected_samples: Option<NOfSamples<SAMPLE_RATE>>,
	) -> Option<(
		InterleavedAudioBuffer<SAMPLE_RATE, N_CH, Vec<f32>>,
		NOfSamples<SAMPLE_RATE>,
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
					let mut out = vec![0.; shared.buffer.len() - *skip];
					if !out.is_empty() {
						shared.buffer.copy_to_slice(*skip, &mut out);
					}
					out
				}),
				collected_samples,
			)
		})
	}

	/// Number of sampling points, regardless of the number of channels.
	#[must_use]
	pub fn n_of_samples(&self) -> NOfSamples<SAMPLE_RATE> {
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
		self.shared
			.with_lock(|shared| shared.input_delay_moving_avg.avg())
	}
}

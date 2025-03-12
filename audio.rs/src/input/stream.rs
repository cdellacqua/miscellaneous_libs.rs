use std::{
	sync::{Arc, Mutex},
	time::Duration,
};

use cpal::{
	traits::{DeviceTrait, StreamTrait},
	Device, Stream, SupportedStreamConfig,
};
use math_utils::moving_avg::MovingAverage;
use mutex_ext::LockExt;
use resource_daemon::ResourceDaemon;

use crate::{
	buffers::InterleavedAudioBuffer, device_provider, AudioStreamBuilderError, AudioStreamError,
	AudioStreamSamplingState, NOfFrames,
};

pub trait StreamListener<const SAMPLE_RATE: usize, const N_CH: usize> {
	fn on_data(&mut self, chunk: InterleavedAudioBuffer<SAMPLE_RATE, N_CH, &[f32]>);
	fn on_error(&mut self, reason: &str);
}

pub struct InputStreamBuilder<const SAMPLE_RATE: usize, const N_CH: usize> {
	device_name: Option<String>,
	listener: Box<dyn StreamListener<SAMPLE_RATE, N_CH> + Send + 'static>,
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> std::fmt::Debug
	for InputStreamBuilder<SAMPLE_RATE, N_CH>
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("InputStreamBuilder")
			.field("device_name", &self.device_name)
			.field("listener", &"<omitted>")
			.finish()
	}
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> InputStreamBuilder<SAMPLE_RATE, N_CH> {
	#[must_use]
	pub const fn new(
		device_name: Option<String>,
		listener: Box<dyn StreamListener<SAMPLE_RATE, N_CH> + Send + 'static>,
	) -> Self {
		Self {
			device_name,
			listener,
		}
	}

	/// Build and start recording the input stream
	///
	/// # Errors
	/// [`AudioStreamBuilderError`]
	pub fn build(self) -> Result<InputStream<SAMPLE_RATE, N_CH>, AudioStreamBuilderError> {
		let (device, config) = device_provider(
			self.device_name.as_deref(),
			crate::IOMode::Input,
			N_CH,
			SAMPLE_RATE,
		)?;

		Ok(InputStream::new(device, config, self.listener))
	}
}

struct StreamState<const SAMPLE_RATE: usize, const N_CH: usize> {
	input_delay_moving_avg: MovingAverage<Duration>,
	listener: Box<dyn StreamListener<SAMPLE_RATE, N_CH> + Send + 'static>,
}

pub struct InputStream<const SAMPLE_RATE: usize, const N_CH: usize> {
	shared: Arc<Mutex<StreamState<SAMPLE_RATE, N_CH>>>,
	stream_daemon: ResourceDaemon<Stream, AudioStreamError>,
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> InputStream<SAMPLE_RATE, N_CH> {
	fn new(
		device: Device,
		config: SupportedStreamConfig,
		listener: Box<dyn StreamListener<SAMPLE_RATE, N_CH> + Send + 'static>,
	) -> Self {
		let shared = Arc::new(Mutex::new({
			StreamState {
				input_delay_moving_avg: MovingAverage::new(10),
				listener,
			}
		}));

		let stream_daemon = ResourceDaemon::new({
			let shared = shared.clone();

			move |quit_signal| {
				device
					.build_input_stream(
						&config.into(),
						{
							let shared = shared.clone();

							move |data: &[f32], info| {
								let input_buffer_frames =
									NOfFrames::<SAMPLE_RATE, N_CH>::new(data.len() / N_CH);

								shared.with_lock_mut(
									|StreamState {
									     ref mut input_delay_moving_avg,
									     listener,
									 }| {
										listener.on_data(InterleavedAudioBuffer::new(data));

										input_delay_moving_avg.push(
											info.timestamp()
												.callback
												.duration_since(&info.timestamp().capture)
												.unwrap_or(Duration::ZERO) + input_buffer_frames
												.to_duration(),
										);
									},
								);
							}
						},
						move |err| {
							quit_signal.dispatch(AudioStreamError::SamplingError(err.to_string()));
							shared.with_lock_mut(|StreamState { listener, .. }| {
								listener.on_error(&err.to_string());
							});
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

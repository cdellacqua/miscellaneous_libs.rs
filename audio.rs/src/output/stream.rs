use std::{
	sync::{Arc, Mutex},
	time::Duration,
};

use cpal::{
	traits::{DeviceTrait, StreamTrait},
	Stream,
};
use math_utils::moving_avg::MovingAverage;
use mutex_ext::LockExt;
use resource_daemon::ResourceDaemon;

use crate::{
	buffers::InterleavedAudioBuffer, device_provider, input::OnErrorCallback,
	AudioStreamBuilderError, AudioStreamError, AudioStreamSamplingState, SampleRate, SamplingCtx,
};

pub type DataProducer = dyn FnMut(InterleavedAudioBuffer<&mut [f32]>) + Send + 'static;

struct StreamState {
	output_delay_moving_avg: MovingAverage<Duration>,
}

pub struct OutputStream {
	sampling_ctx: SamplingCtx,
	shared: Arc<Mutex<StreamState>>,
	stream_daemon: ResourceDaemon<Stream, AudioStreamError>,
}

impl OutputStream {
	/// Build and start recording the input stream
	///
	/// # Errors
	/// [`AudioStreamBuilderError`]
	pub fn new(
		sampling_ctx: SamplingCtx,
		device_name: Option<&str>,
		mut data_producer: Box<DataProducer>,
		mut on_error: Option<Box<OnErrorCallback>>,
	) -> Result<Self, AudioStreamBuilderError> {
		let (device, config) = device_provider(sampling_ctx, device_name, crate::IOMode::Output)?;

		let shared = Arc::new(Mutex::new({
			StreamState {
				output_delay_moving_avg: MovingAverage::new(10),
			}
		}));

		let stream_daemon = ResourceDaemon::new({
			let shared = shared.clone();

			move |quit_signal| {
				device
					.build_output_stream(
						&config.into(),
						{
							let shared = shared.clone();

							move |output: &mut [f32], info| {
								let wrapped = InterleavedAudioBuffer::new(sampling_ctx, output);
								let output_buffer_frames = wrapped.n_of_frames();

								data_producer(wrapped);

								shared.with_lock_mut(
									|StreamState {
									     ref mut output_delay_moving_avg,
									 }| {
										output_delay_moving_avg.push(
											info.timestamp()
												.playback
												.duration_since(&info.timestamp().callback)
												.unwrap_or(Duration::ZERO) + sampling_ctx
												.to_duration(output_buffer_frames),
										);
									},
								);
							}
						},
						move |err| {
							quit_signal.dispatch(AudioStreamError::SamplingError(err.to_string()));
							if let Some(on_error) = on_error.take() {
								on_error(&err.to_string());
							}
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

		Ok(Self {
			sampling_ctx,
			shared,
			stream_daemon,
		})
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
	pub fn sampling_ctx(&self) -> SamplingCtx {
		self.sampling_ctx
	}

	#[must_use]
	pub fn sample_rate(&self) -> SampleRate {
		self.sampling_ctx.sample_rate()
	}

	#[must_use]
	pub fn n_ch(&self) -> usize {
		self.sampling_ctx.n_ch()
	}

	#[must_use]
	pub fn avg_output_delay(&self) -> Duration {
		self.shared
			.with_lock(|shared| shared.output_delay_moving_avg.avg())
	}
}

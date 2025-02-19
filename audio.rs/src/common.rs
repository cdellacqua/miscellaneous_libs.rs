use cpal::{
	traits::{DeviceTrait, HostTrait},
	Device, SampleFormat, SampleRate, SupportedStreamConfig,
};

#[derive(Debug, Clone)]
pub enum AudioStreamSamplingState {
	Sampling,
	Stopped(AudioStreamError),
}

#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioStreamBuilderError {
	#[error("unable to list Input devices")]
	UnableToListDevices,
	#[error("no available device found")]
	NoDeviceFound,
	#[error("no available stream configuration found")]
	NoConfigFound,
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum AudioStreamError {
	#[error("unable to build stream")]
	BuildFailed(String),
	#[error("unable to start stream")]
	StartFailed(String),
	#[error("error while sampling")]
	SamplingError(String),
	#[error("stopped")]
	Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IOMode {
	Input,
	Output,
}

pub(crate) fn device_provider(
	device_name: Option<&str>,
	mode: IOMode,
	n_ch: usize,
	sample_rate: usize,
) -> Result<(Device, SupportedStreamConfig), AudioStreamBuilderError> {
	let device = match mode {
		IOMode::Input => cpal::default_host().input_devices(),
		IOMode::Output => cpal::default_host().output_devices(),
	}
	.map_err(|_| AudioStreamBuilderError::UnableToListDevices)?
	.find(|d| match device_name {
		None => true,
		Some(device_name) => d
			.name()
			.is_ok_and(|candidate_name| candidate_name == device_name),
	})
	.ok_or(AudioStreamBuilderError::NoDeviceFound)?;

	let config = match mode {
		IOMode::Input => device
			.supported_input_configs()
			.map_err(|_| AudioStreamBuilderError::NoConfigFound)?
			.find(|c| c.channels() as usize == n_ch && c.sample_format() == SampleFormat::F32),
		IOMode::Output => device
			.supported_output_configs()
			.map_err(|_| AudioStreamBuilderError::NoConfigFound)?
			.find(|c| c.channels() as usize == n_ch && c.sample_format() == SampleFormat::F32),
	}
	.ok_or(AudioStreamBuilderError::NoConfigFound)?
	.try_with_sample_rate(SampleRate(sample_rate as u32))
	.ok_or(AudioStreamBuilderError::NoConfigFound)?;

	// TODO: normalize everything to f32 and accept any format?

	Ok((device, config))
}

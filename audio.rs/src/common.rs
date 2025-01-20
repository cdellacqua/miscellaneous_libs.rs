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

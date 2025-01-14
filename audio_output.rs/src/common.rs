#[derive(Debug, Clone)]
pub enum SamplingState {
	Sampling,
	Stopped(AudioOutputState),
}

#[derive(thiserror::Error, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum AudioOutputBuilderError {
	#[error("unable to list output devices")]
	UnableToListDevices,
	#[error("no available device found")]
	NoDeviceFound,
	#[error("no available stream configuration found")]
	NoConfigFound,
}

#[derive(thiserror::Error, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum AudioOutputState {
	#[error("unable to build stream")]
	BuildFailed(String),
	#[error("unable to start stream")]
	StartFailed(String),
	#[error("error while sampling")]
	SamplingError(String),
	#[error("stopped")]
	Cancelled,
}

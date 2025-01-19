use std::time::Duration;
pub trait DurationToNOfSamples {
	fn to_n_of_samples(&self, sample_rate: usize) -> usize;
}

pub trait NOfSamplesToDuration {
	/// Note: will convert to microseconds to approximate the number of samples
	fn to_duration(&self, sample_rate: usize) -> Duration;
}

impl DurationToNOfSamples for Duration {
	/// Note: will convert to microseconds to approximate the number of samples
	fn to_n_of_samples(&self, sample_rate: usize) -> usize {
		sample_rate * self.as_micros() as usize / 1_000_000
	}
}

impl NOfSamplesToDuration for usize {
	fn to_duration(&self, sample_rate: usize) -> Duration {
		Duration::from_micros((self * 1_000_000 / sample_rate) as u64)
	}
}

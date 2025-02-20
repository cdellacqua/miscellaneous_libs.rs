use std::{
	ops::{Add, AddAssign, Sub, SubAssign},
	time::Duration,
};

use derive_more::derive::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

/// Note: will convert to microseconds to approximate the number of samples
#[must_use]
pub const fn duration_to_n_of_samples(duration: Duration, sample_rate: usize) -> usize {
	sample_rate * duration.as_micros() as usize / 1_000_000
}

/// Note: will convert to microseconds to approximate the number of samples
#[must_use]
pub const fn n_of_samples_to_duration(samples: usize, sample_rate: usize) -> Duration {
	Duration::from_micros((samples * 1_000_000 / sample_rate) as u64)
}

#[derive(
	Debug,
	Clone,
	Copy,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	Default,
	Hash,
	Add,
	AddAssign,
	Sub,
	SubAssign,
	Div,
	DivAssign,
	Mul,
	MulAssign,
)]
pub struct NOfSamples<const SAMPLE_RATE: usize>(usize);

impl<const SAMPLE_RATE: usize> NOfSamples<SAMPLE_RATE> {
	#[must_use]
	pub const fn new(n_of_samples: usize) -> Self {
		Self(n_of_samples)
	}

	/// Note: will convert to microseconds to approximate the number of samples
	#[must_use]
	pub const fn from_duration(duration: Duration) -> Self {
		Self(duration_to_n_of_samples(duration, SAMPLE_RATE))
	}

	#[must_use]
	pub const fn to_duration(&self) -> Duration {
		n_of_samples_to_duration(self.0, SAMPLE_RATE)
	}

	#[must_use]
	pub const fn inner(&self) -> usize {
		self.0
	}

	#[must_use]
	pub const fn sample_rate(&self) -> usize {
		SAMPLE_RATE
	}
}

impl<const SAMPLE_RATE: usize> From<Duration> for NOfSamples<SAMPLE_RATE> {
	fn from(value: Duration) -> Self {
		Self::from_duration(value)
	}
}

impl<const SAMPLE_RATE: usize> From<usize> for NOfSamples<SAMPLE_RATE> {
	fn from(value: usize) -> Self {
		Self::new(value)
	}
}

impl<const SAMPLE_RATE: usize> From<NOfSamples<SAMPLE_RATE>> for Duration {
	fn from(value: NOfSamples<SAMPLE_RATE>) -> Self {
		value.to_duration()
	}
}

impl<const SAMPLE_RATE: usize> From<NOfSamples<SAMPLE_RATE>> for usize {
	fn from(value: NOfSamples<SAMPLE_RATE>) -> Self {
		value.0
	}
}

impl<const SAMPLE_RATE: usize> Add<usize> for NOfSamples<SAMPLE_RATE> {
	type Output = Self;

	fn add(self, rhs: usize) -> Self::Output {
		(self.0 + rhs).into()
	}
}

impl<const SAMPLE_RATE: usize> AddAssign<usize> for NOfSamples<SAMPLE_RATE> {
	fn add_assign(&mut self, rhs: usize) {
		self.0 += rhs;
	}
}

impl<const SAMPLE_RATE: usize> Sub<usize> for NOfSamples<SAMPLE_RATE> {
	type Output = Self;

	fn sub(self, rhs: usize) -> Self::Output {
		(self.0 - rhs).into()
	}
}

impl<const SAMPLE_RATE: usize> SubAssign<usize> for NOfSamples<SAMPLE_RATE> {
	fn sub_assign(&mut self, rhs: usize) {
		self.0 -= rhs;
	}
}

#[cfg(test)]
mod tests {
	use std::time::Duration;

	use super::*;

	#[test]
	fn test_duration_to_n_of_samples() {
		assert_eq!(
			duration_to_n_of_samples(Duration::from_millis(100), 44100),
			4410
		);
		assert_eq!(
			duration_to_n_of_samples(Duration::from_secs(1), 44100),
			44100
		);
		assert_eq!(
			duration_to_n_of_samples(Duration::from_secs(2), 44100),
			2 * 44100
		);
	}

	#[test]
	fn test_n_of_samples_to_duration() {
		assert_eq!(
			n_of_samples_to_duration(4410, 44100),
			Duration::from_millis(100)
		);
		assert_eq!(
			n_of_samples_to_duration(44100, 44100),
			Duration::from_secs(1)
		);
		assert_eq!(
			n_of_samples_to_duration(2 * 44100, 44100),
			Duration::from_secs(2)
		);
	}
}

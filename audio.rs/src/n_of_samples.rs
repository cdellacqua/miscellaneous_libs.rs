use std::{
	ops::{Div, DivAssign, Mul, MulAssign},
	time::Duration,
};

use derive_more::derive::{Add, AddAssign, Deref, DerefMut, Sub, SubAssign};

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
	Deref,
	DerefMut,
	Default,
	Hash,
	Add,
	AddAssign,
	Sub,
	SubAssign,
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

// impl<const SAMPLE_RATE: usize> Add for NOfSamples<SAMPLE_RATE> {
// 	type Output = NOfSamples<SAMPLE_RATE>;

// 	fn add(self, rhs: Self) -> Self::Output {
// 		Self::new(self.0 + rhs.0)
// 	}
// }

// impl<const SAMPLE_RATE: usize> AddAssign for NOfSamples<SAMPLE_RATE> {
// 	fn add_assign(&mut self, rhs: Self) {
// 		self.0 += rhs.0;
// 	}
// }

// impl<const SAMPLE_RATE: usize> Sub for NOfSamples<SAMPLE_RATE> {
// 	type Output = NOfSamples<SAMPLE_RATE>;

// 	fn sub(self, rhs: Self) -> Self::Output {
// 		Self::new(self.0 - rhs.0)
// 	}
// }

// impl<const SAMPLE_RATE: usize> SubAssign for NOfSamples<SAMPLE_RATE> {
// 	fn sub_assign(&mut self, rhs: Self) {
// 		self.0 -= rhs.0;
// 	}
// }

// impl<const SAMPLE_RATE: usize> Div for NOfSamples<SAMPLE_RATE> {
// 	type Output = NOfSamples<SAMPLE_RATE>;

// 	fn div(self, rhs: Self) -> Self::Output {
// 		Self::new(self.0 / rhs.0)
// 	}
// }

// impl<const SAMPLE_RATE: usize> DivAssign for NOfSamples<SAMPLE_RATE> {
// 	fn div_assign(&mut self, rhs: Self) {
// 		self.0 /= rhs.0;
// 	}
// }

// impl<const SAMPLE_RATE: usize> Mul for NOfSamples<SAMPLE_RATE> {
// 	type Output = NOfSamples<SAMPLE_RATE>;

// 	fn mul(self, rhs: Self) -> Self::Output {
// 		Self::new(self.0 * rhs.0)
// 	}
// }

impl<const SAMPLE_RATE: usize> Mul<usize> for NOfSamples<SAMPLE_RATE> {
	type Output = NOfSamples<SAMPLE_RATE>;

	fn mul(self, rhs: usize) -> Self::Output {
		Self::new(self.0 * rhs)
	}
}

impl<const SAMPLE_RATE: usize> MulAssign<usize> for NOfSamples<SAMPLE_RATE> {
	fn mul_assign(&mut self, rhs: usize) {
		self.0 *= rhs;
	}
}

impl<const SAMPLE_RATE: usize> Div<usize> for NOfSamples<SAMPLE_RATE> {
	type Output = NOfSamples<SAMPLE_RATE>;

	fn div(self, rhs: usize) -> Self::Output {
		Self::new(self.0 / rhs)
	}
}

impl<const SAMPLE_RATE: usize> DivAssign<usize> for NOfSamples<SAMPLE_RATE> {
	fn div_assign(&mut self, rhs: usize) {
		self.0 /= rhs;
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

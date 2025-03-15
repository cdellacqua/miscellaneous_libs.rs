use std::{
	ops::{Add, AddAssign, Sub, SubAssign},
	time::Duration,
};

use derive_more::derive::{
	Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign,
};

/// Note: will convert to microseconds to approximate the number of frames
#[must_use]
pub const fn duration_to_n_of_frames(duration: Duration, sample_rate: usize) -> usize {
	sample_rate * duration.as_micros() as usize / 1_000_000
}

/// Note: will convert to microseconds to approximate the number of frames
#[must_use]
pub const fn n_of_frames_to_duration(frames: usize, sample_rate: usize) -> Duration {
	Duration::from_micros((frames * 1_000_000 / sample_rate) as u64)
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
	Rem,
	RemAssign,
)]
pub struct NOfFrames<const SAMPLE_RATE: usize, const N_CH: usize>(usize);

impl<const SAMPLE_RATE: usize, const N_CH: usize> NOfFrames<SAMPLE_RATE, N_CH> {
	#[must_use]
	pub const fn new(n_of_frames: usize) -> Self {
		Self(n_of_frames)
	}

	#[must_use]
	pub fn from_n_of_samples(n_of_samples: usize) -> Self {
		debug_assert_eq!(
			n_of_samples % N_CH,
			0,
			"provided n_of_samples ({n_of_samples}) is not a multiple of N_CH {N_CH}"
		);
		Self(n_of_samples / N_CH)
	}

	/// Note: will convert to microseconds to approximate the number of frames
	#[must_use]
	pub const fn from_duration(duration: Duration) -> Self {
		Self(duration_to_n_of_frames(duration, SAMPLE_RATE))
	}

	#[must_use]
	pub const fn to_duration(&self) -> Duration {
		n_of_frames_to_duration(self.0, SAMPLE_RATE)
	}

	#[must_use]
	pub const fn inner(&self) -> usize {
		self.0
	}

	#[must_use]
	pub const fn sample_rate(&self) -> usize {
		SAMPLE_RATE
	}

	#[must_use]
	pub const fn n_of_channels(&self) -> usize {
		N_CH
	}

	/// Get the product of the number of frames and the number of channels, resulting
	/// in the number of sampling points. This is the number you would usually use to
	/// allocate a raw audio buffer.
	#[must_use]
	pub const fn n_of_samples(&self) -> usize {
		N_CH * self.0
	}
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> From<Duration> for NOfFrames<SAMPLE_RATE, N_CH> {
	fn from(value: Duration) -> Self {
		Self::from_duration(value)
	}
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> From<usize> for NOfFrames<SAMPLE_RATE, N_CH> {
	fn from(value: usize) -> Self {
		Self::new(value)
	}
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> From<NOfFrames<SAMPLE_RATE, N_CH>> for Duration {
	fn from(value: NOfFrames<SAMPLE_RATE, N_CH>) -> Self {
		value.to_duration()
	}
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> From<NOfFrames<SAMPLE_RATE, N_CH>> for usize {
	fn from(value: NOfFrames<SAMPLE_RATE, N_CH>) -> Self {
		value.0
	}
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> Add<usize> for NOfFrames<SAMPLE_RATE, N_CH> {
	type Output = Self;

	fn add(self, rhs: usize) -> Self::Output {
		(self.0 + rhs).into()
	}
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> AddAssign<usize>
	for NOfFrames<SAMPLE_RATE, N_CH>
{
	fn add_assign(&mut self, rhs: usize) {
		self.0 += rhs;
	}
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> Sub<usize> for NOfFrames<SAMPLE_RATE, N_CH> {
	type Output = Self;

	fn sub(self, rhs: usize) -> Self::Output {
		(self.0 - rhs).into()
	}
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> SubAssign<usize>
	for NOfFrames<SAMPLE_RATE, N_CH>
{
	fn sub_assign(&mut self, rhs: usize) {
		self.0 -= rhs;
	}
}

#[cfg(test)]
mod tests {
	use std::time::Duration;

	use super::*;

	#[test]
	fn test_duration_to_n_of_frames() {
		assert_eq!(
			duration_to_n_of_frames(Duration::from_millis(100), 44100),
			4410
		);
		assert_eq!(
			duration_to_n_of_frames(Duration::from_secs(1), 44100),
			44100
		);
		assert_eq!(
			duration_to_n_of_frames(Duration::from_secs(2), 44100),
			2 * 44100
		);
	}

	#[test]
	fn test_n_of_frames_to_duration() {
		assert_eq!(
			n_of_frames_to_duration(4410, 44100),
			Duration::from_millis(100)
		);
		assert_eq!(
			n_of_frames_to_duration(44100, 44100),
			Duration::from_secs(1)
		);
		assert_eq!(
			n_of_frames_to_duration(2 * 44100, 44100),
			Duration::from_secs(2)
		);
	}
}

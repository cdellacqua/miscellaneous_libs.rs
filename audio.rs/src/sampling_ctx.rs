use std::time::Duration;

use crate::{NOfFrames, SampleRate};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SamplingCtx {
	sample_rate: SampleRate,
	n_ch: usize,
}

impl SamplingCtx {
	#[must_use]
	pub const fn new(sample_rate: SampleRate, n_ch: usize) -> Self {
		Self { sample_rate, n_ch }
	}

	/// The number of samples (or rather, frames) per second
	#[must_use]
	pub const fn sample_rate(&self) -> SampleRate {
		self.sample_rate
	}

	/// The number of channels
	#[must_use]
	pub const fn n_ch(&self) -> usize {
		self.n_ch
	}

	#[must_use]
	pub fn samples_to_frames(&self, n_of_samples: usize) -> NOfFrames {
		debug_assert_eq!(
			n_of_samples % self.n_ch,
			0,
			"provided n_of_samples ({n_of_samples}) is not a multiple of N_CH {}",
			self.n_ch
		);
		NOfFrames(n_of_samples / self.n_ch)
	}

	/// Note: will convert to microseconds to approximate the number of frames
	#[must_use]
	pub const fn to_n_of_frames(&self, duration: Duration) -> NOfFrames {
		NOfFrames(self.sample_rate.0 * duration.as_micros() as usize / 1_000_000)
	}

	/// Note: will convert to microseconds to approximate the number of frames
	#[must_use]
	pub const fn to_duration(&self, n_of_frames: NOfFrames) -> Duration {
		Duration::from_micros((n_of_frames.0 * 1_000_000 / self.sample_rate.0) as u64)
	}

	/// Get the product of the number of frames and the number of channels, resulting
	/// in the number of sampling points. This is the number you would usually use to
	/// allocate a raw audio buffer.
	#[must_use]
	pub const fn n_of_samples(&self, n_of_frames: NOfFrames) -> usize {
		self.n_ch * n_of_frames.0
	}
}

#[cfg(test)]
mod tests {
	use std::time::Duration;

	use super::*;

	#[test]
	fn test_duration_to_n_of_frames() {
		let sampling_ctx = SamplingCtx::new(44100.into(), 2);
		assert_eq!(
			sampling_ctx.to_n_of_frames(Duration::from_millis(100)).0,
			4410
		);
		assert_eq!(sampling_ctx.to_n_of_frames(Duration::from_secs(1)).0, 44100);
		assert_eq!(
			sampling_ctx.to_n_of_frames(Duration::from_secs(2)).0,
			2 * 44100
		);
	}

	#[test]
	fn test_n_of_frames_to_duration() {
		let sampling_ctx = SamplingCtx::new(44100.into(), 2);
		assert_eq!(
			sampling_ctx.to_duration(NOfFrames(4410)),
			Duration::from_millis(100)
		);
		assert_eq!(
			sampling_ctx.to_duration(NOfFrames(44100)),
			Duration::from_secs(1)
		);
		assert_eq!(
			sampling_ctx.to_duration(NOfFrames(2 * 44100)),
			Duration::from_secs(2)
		);
	}
}

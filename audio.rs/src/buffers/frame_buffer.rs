#![allow(clippy::cast_precision_loss)]

use std::{
	borrow::{Borrow, BorrowMut},
	ops::{Deref, DerefMut, Index, IndexMut},
};

#[derive(Debug, Clone, Copy)]
pub struct AudioFrame<const N_CH: usize, Samples: Borrow<[f32; N_CH]>>(Samples);

impl<const N_CH: usize, Samples: Borrow<[f32; N_CH]>> AudioFrame<N_CH, Samples> {
	/// # Panics
	/// - if the buffer size doesn't correspond to the number of channels.
	#[must_use]
	pub fn new(samples: Samples) -> Self {
		assert_eq!(samples.borrow().len(), N_CH);

		AudioFrame(samples)
	}

	#[must_use]
	pub fn copied(&self) -> AudioFrame<N_CH, [f32; N_CH]> {
		self.cloned()
	}

	#[must_use]
	pub fn samples(&self) -> &[f32; N_CH] {
		self.0.borrow()
	}

	#[must_use]
	pub fn cloned(&self) -> AudioFrame<N_CH, [f32; N_CH]> {
		AudioFrame(*self.0.borrow())
	}

	#[must_use]
	pub fn to_mono(&self) -> f32 {
		let samples: &[f32; N_CH] = self.0.borrow();
		// Values are usually from -1..1 and channels are usually single digit numbers,
		// the sum shouldn't overflow.
		samples.iter().sum::<f32>() / (samples.len() as f32)
	}

	#[must_use]
	pub fn n_of_channels(&self) -> usize {
		N_CH
	}
}

impl<const N_CH: usize, Samples: BorrowMut<[f32; N_CH]>> AudioFrame<N_CH, Samples> {
	#[must_use]
	pub fn samples_mut(&mut self) -> &mut [f32; N_CH] {
		self.0.borrow_mut()
	}
}

impl<const N_CH: usize, A: Borrow<[f32; N_CH]>, B: Borrow<[f32; N_CH]>>
	PartialEq<AudioFrame<N_CH, B>> for AudioFrame<N_CH, A>
{
	fn eq(&self, other: &AudioFrame<N_CH, B>) -> bool {
		self.0.borrow() == other.0.borrow()
	}
}

impl<const N_CH: usize, A: Borrow<[f32; N_CH]>, B: Borrow<[f32; N_CH]>>
	PartialOrd<AudioFrame<N_CH, B>> for AudioFrame<N_CH, A>
{
	fn partial_cmp(&self, other: &AudioFrame<N_CH, B>) -> Option<std::cmp::Ordering> {
		self.0.borrow().partial_cmp(other.0.borrow())
	}
}

impl<const N_CH: usize, Samples: Borrow<[f32; N_CH]>> Index<usize> for AudioFrame<N_CH, Samples> {
	type Output = f32;

	fn index(&self, index: usize) -> &Self::Output {
		&self.0.borrow()[index]
	}
}

impl<const N_CH: usize, Samples: BorrowMut<[f32; N_CH]>> IndexMut<usize>
	for AudioFrame<N_CH, Samples>
{
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.0.borrow_mut()[index]
	}
}

impl<const N_CH: usize, Samples: Borrow<[f32; N_CH]>> Deref for AudioFrame<N_CH, Samples> {
	type Target = [f32; N_CH];

	fn deref(&self) -> &Self::Target {
		self.0.borrow()
	}
}

impl<const N_CH: usize, Samples: BorrowMut<[f32; N_CH]>> DerefMut for AudioFrame<N_CH, Samples> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.0.borrow_mut()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_copied() {
		let snapshot = AudioFrame::new(&[1_f32, 2_f32]);
		let _a: AudioFrame<2, [f32; 2]> = snapshot.copied();
	}
}

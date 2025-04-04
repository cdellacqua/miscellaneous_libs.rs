#![allow(clippy::cast_precision_loss)]

use std::{
	borrow::{Borrow, BorrowMut},
	ops::{Index, IndexMut},
};

#[derive(Debug)]
pub struct AudioFrame<Samples: Borrow<[f32]>>(Samples);

impl<Samples: Borrow<[f32]>> AudioFrame<Samples> {
	#[must_use]
	pub fn new(samples: Samples) -> Self {
		AudioFrame(samples)
	}

	#[must_use]
	pub fn samples(&self) -> &[f32] {
		self.0.borrow()
	}

	#[must_use]
	pub fn cloned(&self) -> AudioFrame<Vec<f32>> {
		AudioFrame(self.0.borrow().to_vec())
	}

	#[must_use]
	pub fn to_mono(&self) -> f32 {
		let samples: &[f32] = self.0.borrow();

		if samples.len() == 1 {
			samples[0]
		} else {
			// Values are usually from -1..1 and channels are usually single digit numbers,
			// the sum shouldn't overflow.
			samples.iter().sum::<f32>() / (samples.len() as f32)
		}
	}

	#[must_use]
	pub fn n_ch(&self) -> usize {
		self.samples().len()
	}
}

impl<Samples: BorrowMut<[f32]>> AudioFrame<Samples> {
	#[must_use]
	pub fn samples_mut(&mut self) -> &mut [f32] {
		self.0.borrow_mut()
	}
}

impl<A: Borrow<[f32]>, B: Borrow<[f32]>> PartialEq<AudioFrame<B>> for AudioFrame<A> {
	fn eq(&self, other: &AudioFrame<B>) -> bool {
		self.0.borrow() == other.0.borrow()
	}
}

impl<A: Borrow<[f32]>, B: Borrow<[f32]>> PartialOrd<AudioFrame<B>> for AudioFrame<A> {
	fn partial_cmp(&self, other: &AudioFrame<B>) -> Option<std::cmp::Ordering> {
		self.0.borrow().partial_cmp(other.0.borrow())
	}
}

impl<Samples: Borrow<[f32]>> Index<usize> for AudioFrame<Samples> {
	type Output = f32;

	fn index(&self, index: usize) -> &Self::Output {
		&self.0.borrow()[index]
	}
}

impl<Samples: BorrowMut<[f32]>> IndexMut<usize> for AudioFrame<Samples> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.0.borrow_mut()[index]
	}
}

impl<Samples: Borrow<[f32]>> AsRef<[f32]> for AudioFrame<Samples> {
	fn as_ref(&self) -> &[f32] {
		self.0.borrow()
	}
}

impl<Samples: BorrowMut<[f32]>> AsMut<[f32]> for AudioFrame<Samples> {
	fn as_mut(&mut self) -> &mut [f32] {
		self.0.borrow_mut()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_copied() {
		let snapshot = AudioFrame::new([1_f32, 2_f32].as_slice());
		let _a: AudioFrame<Vec<f32>> = snapshot.cloned();
	}
}

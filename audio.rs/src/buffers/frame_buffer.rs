#![allow(clippy::cast_precision_loss)]

use std::{
	borrow::{Borrow, BorrowMut},
	ops::{Deref, DerefMut, Index, IndexMut},
};

use super::{NOfChannels, ToMono};

#[derive(Debug, Clone, Copy)]
pub struct AudioFrame<const N_CH: usize, Samples: Borrow<[f32]>>(Samples);

impl<const N_CH: usize, Samples: Borrow<[f32]>> AudioFrame<N_CH, Samples> {
	pub fn new(samples: Samples) -> Self {
		AudioFrame(samples)
	}

	pub fn to_owned(&self) -> AudioFrame<N_CH, Vec<f32>> {
		AudioFrame(self.0.borrow().to_owned())
	}

	pub fn to_mono(&self) -> f32 {
		let samples: &[f32] = self.0.borrow();
		// Values are usually from -1..1 and channels are usually single digit numbers,
		// the sum shouldn't overflow.
		samples.iter().sum::<f32>() / (samples.len() as f32)
	}

	pub fn n_of_channels(&self) -> usize {
		N_CH
	}
}

impl<const N_CH: usize, Samples: Borrow<[f32]>> ToMono for AudioFrame<N_CH, Samples> {
	type Target = f32;

	fn to_mono(&self) -> Self::Target {
		self.to_mono()
	}
}

impl<const N_CH: usize, Samples: Borrow<[f32]>> NOfChannels for AudioFrame<N_CH, Samples> {
	fn n_of_channels(&self) -> usize {
		N_CH
	}
}

impl<const N_CH: usize, A: Borrow<[f32]>, B: Borrow<[f32]>> PartialEq<AudioFrame<N_CH, B>>
	for AudioFrame<N_CH, A>
{
	fn eq(&self, other: &AudioFrame<N_CH, B>) -> bool {
		self.0.borrow() == other.0.borrow()
	}
}

impl<const N_CH: usize, A: Borrow<[f32]>, B: Borrow<[f32]>> PartialOrd<AudioFrame<N_CH, B>>
	for AudioFrame<N_CH, A>
{
	fn partial_cmp(&self, other: &AudioFrame<N_CH, B>) -> Option<std::cmp::Ordering> {
		self.0.borrow().partial_cmp(other.0.borrow())
	}
}

impl<const N_CH: usize, Samples: Borrow<[f32]>> Index<usize> for AudioFrame<N_CH, Samples> {
	type Output = f32;

	fn index(&self, index: usize) -> &Self::Output {
		&self.0.borrow()[index]
	}
}

impl<const N_CH: usize, Samples: BorrowMut<[f32]>> IndexMut<usize> for AudioFrame<N_CH, Samples> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.0.borrow_mut()[index]
	}
}

impl<const N_CH: usize, Samples: Borrow<[f32]>> Deref for AudioFrame<N_CH, Samples> {
	type Target = [f32];

	fn deref(&self) -> &Self::Target {
		self.0.borrow()
	}
}

impl<const N_CH: usize, Samples: BorrowMut<[f32]>> DerefMut for AudioFrame<N_CH, Samples> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.0.borrow_mut()
	}
}

pub trait AudioNFrameTrait<const N_CH: usize>:
	ToMono<Target = f32>
	+ NOfChannels
	+ Index<usize, Output = f32>
	+ Deref<Target = [f32]>
	+ Send
	+ Sync
{
}

pub trait AudioNFrameTraitMut<const N_CH: usize>:
	AudioNFrameTrait<N_CH> + IndexMut<usize, Output = f32> + DerefMut<Target = [f32]>
{
}

// TODO: generic instead of f32
// TODO: better naming?
pub trait AudioFrameTrait:
	ToMono<Target = f32>
	+ NOfChannels
	+ Index<usize, Output = f32>
	+ Deref<Target = [f32]>
	+ Sync
	+ Send
{
}
// TODO: better naming?
pub trait AudioFrameTraitMut:
	AudioFrameTrait + IndexMut<usize, Output = f32> + DerefMut<Target = [f32]>
{
}

impl<const N_CH: usize, Samples: Borrow<[f32]> + Send + Sync> AudioNFrameTrait<N_CH>
	for AudioFrame<N_CH, Samples>
{
}

impl<const N_CH: usize, Samples: BorrowMut<[f32]> + Send + Sync> AudioNFrameTraitMut<N_CH>
	for AudioFrame<N_CH, Samples>
{
}

impl<const N_CH: usize, Samples: Borrow<[f32]> + Send + Sync> AudioFrameTrait
	for AudioFrame<N_CH, Samples>
{
}

impl<const N_CH: usize, Samples: BorrowMut<[f32]> + Send + Sync> AudioFrameTraitMut
	for AudioFrame<N_CH, Samples>
{
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_to_owned() {
		let snapshot = AudioFrame::new(&[1_f32, 2_f32] as &[f32]);
		let _a: AudioFrame<2, Vec<f32>> = snapshot.to_owned();
	}
}

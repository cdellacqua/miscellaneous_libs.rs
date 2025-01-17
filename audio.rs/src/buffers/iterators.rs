use std::borrow::{Borrow, BorrowMut};

use super::{AudioFrame, InterleavedAudioBuffer};

// #region immutable
#[derive(Debug, Clone)]
pub struct InterleavedAudioBufferIter<'a, const N_CH: usize, Buffer: Borrow<[f32]>> {
	i: usize,
	max: usize,
	interleaved_samples: &'a InterleavedAudioBuffer<N_CH, Buffer>,
}

impl<'a, const N_CH: usize, Buffer: Borrow<[f32]>> InterleavedAudioBufferIter<'a, N_CH, Buffer> {
	pub(crate) fn new(interleaved_samples: &'a InterleavedAudioBuffer<N_CH, Buffer>) -> Self {
		Self {
			i: 0,
			max: interleaved_samples.n_of_frames(),
			interleaved_samples,
		}
	}
}

impl<'a, const N_CH: usize, Buffer: Borrow<[f32]>> Iterator
	for InterleavedAudioBufferIter<'a, N_CH, Buffer>
{
	type Item = AudioFrame<N_CH, &'a [f32]>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.i < self.max {
			let frame = self.interleaved_samples.at(self.i);
			self.i += 1;

			Some(frame)
		} else {
			None
		}
	}
}
// #endregion

// #region mutable
#[derive(Debug)]
pub struct InterleavedAudioBufferIterMut<'a, const N_CH: usize, Buffer: BorrowMut<[f32]>> {
	i: usize,
	max: usize,
	interleaved_samples: &'a mut InterleavedAudioBuffer<N_CH, Buffer>,
}

impl<'a, const N_CH: usize, Buffer: BorrowMut<[f32]>>
	InterleavedAudioBufferIterMut<'a, N_CH, Buffer>
{
	pub(crate) fn new(interleaved_samples: &'a mut InterleavedAudioBuffer<N_CH, Buffer>) -> Self {
		Self {
			i: 0,
			max: interleaved_samples.n_of_frames(),
			interleaved_samples,
		}
	}
}

impl<'a, const N_CH: usize, Buffer: BorrowMut<[f32]>> Iterator
	for InterleavedAudioBufferIterMut<'a, N_CH, Buffer>
{
	type Item = AudioFrame<N_CH, &'a mut [f32]>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.i < self.max {
			let slice: &mut [f32] =
				&mut self.interleaved_samples.raw_buffer_mut()[self.i * N_CH..(self.i + 1) * N_CH];

			// SAFETY:
			// - array size invariant guaranteed by `assert_eq` in the constructor of the buffer
			// - lifetime compatibility guaranteed by compatible borrows.
			let frame: AudioFrame<N_CH, &mut [f32]> = AudioFrame::new(unsafe {
				std::slice::from_raw_parts_mut(slice.as_mut_ptr(), N_CH)
			});

			self.i += 1;

			Some(frame)
		} else {
			None
		}
	}
}
// #endregion

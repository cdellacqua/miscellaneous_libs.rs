use core::slice;
use std::borrow::{Borrow, BorrowMut};

use super::{AudioFrame, InterleavedAudioBuffer};

// #region immutable
#[derive(Debug, Clone)]
pub struct InterleavedAudioBufferIter<'a, Buffer: Borrow<[f32]>> {
	i: usize,
	max: usize,
	interleaved_frames: &'a InterleavedAudioBuffer<Buffer>,
}

impl<'a, Buffer: Borrow<[f32]>> InterleavedAudioBufferIter<'a, Buffer> {
	pub(crate) fn new(interleaved_frames: &'a InterleavedAudioBuffer<Buffer>) -> Self {
		Self {
			i: 0,
			max: interleaved_frames.n_of_frames().0,
			interleaved_frames,
		}
	}
}

impl<'a, Buffer: Borrow<[f32]>> Iterator for InterleavedAudioBufferIter<'a, Buffer> {
	type Item = AudioFrame<&'a [f32]>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.i < self.max {
			let frame = self.interleaved_frames.at(self.i);
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
pub struct InterleavedAudioBufferIterMut<'a, Buffer: BorrowMut<[f32]>> {
	i: usize,
	max: usize,
	interleaved_frames: &'a mut InterleavedAudioBuffer<Buffer>,
}

impl<'a, Buffer: BorrowMut<[f32]>> InterleavedAudioBufferIterMut<'a, Buffer> {
	pub(crate) fn new(interleaved_frames: &'a mut InterleavedAudioBuffer<Buffer>) -> Self {
		Self {
			i: 0,
			max: interleaved_frames.n_of_frames().0,
			interleaved_frames,
		}
	}
}

impl<'a, Buffer: BorrowMut<[f32]>> Iterator for InterleavedAudioBufferIterMut<'a, Buffer> {
	type Item = AudioFrame<&'a mut [f32]>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.i < self.max {
			let frame = self
				.interleaved_frames
				.at_mut(self.i)
				.samples_mut()
				.as_mut_ptr();

			self.i += 1;

			// SAFETY: the iterator has an exclusive reference to the underlying buffer and
			// it's giving out mutable references to disjoint regions of memory.
			Some(unsafe {
				AudioFrame::new(slice::from_raw_parts_mut(
					frame,
					self.interleaved_frames.n_ch(),
				))
			})
		} else {
			None
		}
	}
}
// #endregion

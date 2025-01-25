use std::borrow::{Borrow, BorrowMut};

use super::{AudioFrame, InterleavedAudioBuffer};

// #region immutable
#[derive(Debug, Clone)]
pub struct InterleavedAudioBufferIter<
	'a,
	const SAMPLE_RATE: usize,
	const N_CH: usize,
	Buffer: Borrow<[f32]>,
> {
	i: usize,
	max: usize,
	interleaved_samples: &'a InterleavedAudioBuffer<SAMPLE_RATE, N_CH, Buffer>,
}

impl<'a, const SAMPLE_RATE: usize, const N_CH: usize, Buffer: Borrow<[f32]>>
	InterleavedAudioBufferIter<'a, SAMPLE_RATE, N_CH, Buffer>
{
	pub(crate) fn new(
		interleaved_samples: &'a InterleavedAudioBuffer<SAMPLE_RATE, N_CH, Buffer>,
	) -> Self {
		Self {
			i: 0,
			max: *interleaved_samples.n_of_samples(),
			interleaved_samples,
		}
	}
}

impl<'a, const SAMPLE_RATE: usize, const N_CH: usize, Buffer: Borrow<[f32]>> Iterator
	for InterleavedAudioBufferIter<'a, SAMPLE_RATE, N_CH, Buffer>
{
	type Item = AudioFrame<N_CH, &'a [f32; N_CH]>;

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
pub struct InterleavedAudioBufferIterMut<
	'a,
	const SAMPLE_RATE: usize,
	const N_CH: usize,
	Buffer: BorrowMut<[f32]>,
> {
	i: usize,
	max: usize,
	interleaved_samples: &'a mut InterleavedAudioBuffer<SAMPLE_RATE, N_CH, Buffer>,
}

impl<'a, const SAMPLE_RATE: usize, const N_CH: usize, Buffer: BorrowMut<[f32]>>
	InterleavedAudioBufferIterMut<'a, SAMPLE_RATE, N_CH, Buffer>
{
	pub(crate) fn new(
		interleaved_samples: &'a mut InterleavedAudioBuffer<SAMPLE_RATE, N_CH, Buffer>,
	) -> Self {
		Self {
			i: 0,
			max: *interleaved_samples.n_of_samples(),
			interleaved_samples,
		}
	}
}

impl<'a, const SAMPLE_RATE: usize, const N_CH: usize, Buffer: BorrowMut<[f32]>> Iterator
	for InterleavedAudioBufferIterMut<'a, SAMPLE_RATE, N_CH, Buffer>
{
	type Item = AudioFrame<N_CH, &'a mut [f32; N_CH]>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.i < self.max {
			// SAFETY:
			// - array size invariant guaranteed by `assert_eq` in the constructor of the buffer
			// - lifetime compatibility guaranteed by compatible borrows.
			let frame: AudioFrame<N_CH, &mut [f32; N_CH]> = AudioFrame::new(unsafe {
				&mut *self.interleaved_samples.raw_buffer_mut()[self.i * N_CH..(self.i + 1) * N_CH]
					.as_mut_ptr()
					.cast::<[_; N_CH]>()
			});

			self.i += 1;

			Some(frame)
		} else {
			None
		}
	}
}
// #endregion

// #region owned
#[derive(Debug, Clone)]
pub struct InterleavedAudioBufferIterOwned<
	const SAMPLE_RATE: usize,
	const N_CH: usize,
	Buffer: Borrow<[f32]>,
> {
	i: usize,
	max: usize,
	interleaved_samples: InterleavedAudioBuffer<SAMPLE_RATE, N_CH, Buffer>,
}

impl<const SAMPLE_RATE: usize, const N_CH: usize, Buffer: Borrow<[f32]>>
	InterleavedAudioBufferIterOwned<SAMPLE_RATE, N_CH, Buffer>
{
	pub(crate) fn new(
		interleaved_samples: InterleavedAudioBuffer<SAMPLE_RATE, N_CH, Buffer>,
	) -> Self {
		Self {
			i: 0,
			max: *interleaved_samples.borrow().n_of_samples(),
			interleaved_samples,
		}
	}
}

impl<const SAMPLE_RATE: usize, const N_CH: usize, Buffer: Borrow<[f32]>> Iterator
	for InterleavedAudioBufferIterOwned<SAMPLE_RATE, N_CH, Buffer>
{
	type Item = AudioFrame<N_CH, [f32; N_CH]>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.i < self.max {
			let frame = self.interleaved_samples.at(self.i).cloned();

			self.i += 1;

			Some(frame)
		} else {
			None
		}
	}
}
// #endregion

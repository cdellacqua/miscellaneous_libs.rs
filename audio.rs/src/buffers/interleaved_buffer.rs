use std::{
	borrow::{Borrow, BorrowMut},
	ops::{Deref, DerefMut},
};

use super::{
	frame_buffer::{AudioFrame, AudioFrameTrait},
	InterleavedAudioBufferIter, InterleavedAudioBufferIterMut, NOfChannels, ToMono,
};

#[derive(Debug, Clone)]
pub struct InterleavedAudioBuffer<const N_CH: usize, Buffer: Borrow<[f32]>> {
	raw_buffer: Buffer,
}

impl<const N_CH: usize, Buffer: Borrow<[f32]>> InterleavedAudioBuffer<N_CH, Buffer> {
	/// Creates a new [`InterleavedAudioBuffer`].
	///
	/// # Panics
	/// - if the buffer size is not a multiple of the number of channels.
	#[must_use]
	pub fn new(raw_buffer: Buffer) -> Self {
		assert_eq!(
			raw_buffer.borrow().len() % N_CH,
			0,
			"buffer size must be a multiple of the number of channels"
		);
		Self { raw_buffer }
	}

	#[must_use]
	#[allow(clippy::missing_panics_doc)] // REASON: invariant guaranteed by `assert_eq` in the constructor
	pub fn at(&self, index: usize) -> AudioFrame<N_CH, &[f32]> {
		AudioFrame::new(
			self.raw_buffer.borrow()[index * N_CH..(index + 1) * N_CH]
				.try_into()
				.unwrap(),
		)
	}

	#[must_use]
	pub fn iter(&self) -> InterleavedAudioBufferIter<N_CH, Buffer> {
		InterleavedAudioBufferIter::new(self)
	}

	#[must_use]
	pub fn into_raw(self) -> (usize, Buffer) {
		(N_CH, self.raw_buffer)
	}

	/// The number of frames corresponds to the number of sampling points in time, regardless of the number
	/// of channels.
	#[must_use]
	pub fn n_of_frames(&self) -> usize {
		self.raw_buffer.borrow().len() / N_CH
	}

	/// Converts this interleaved collection to a raw buffer containing the samples of a mono track.
	/// Samples in the mono track are the average of all the channel samples for each point in time.
	#[must_use]
	pub fn to_mono(&self) -> Vec<f32> {
		self.iter().map(|frame| frame.to_mono()).collect()
	}

	#[must_use]
	pub fn n_of_channels(&self) -> usize {
		N_CH
	}

	#[must_use]
	pub fn raw_buffer(&self) -> &[f32] {
		self.raw_buffer.borrow()
	}
}

impl<const N_CH: usize, Buffer: BorrowMut<[f32]>> InterleavedAudioBuffer<N_CH, Buffer> {
	/// # Panics
	/// - if the index is out of bound.
	#[must_use]
	pub fn at_mut(&mut self, index: usize) -> AudioFrame<N_CH, &mut [f32]> {
		assert!(index < self.n_of_frames());

		let slice: &mut [f32] = &mut self.raw_buffer.borrow_mut()[index * N_CH..(index + 1) * N_CH];

		AudioFrame::new(slice)
	}

	#[must_use]
	pub fn iter_mut(&mut self) -> InterleavedAudioBufferIterMut<N_CH, Buffer> {
		InterleavedAudioBufferIterMut::new(self)
	}

	#[must_use]
	pub fn raw_buffer_mut(&mut self) -> &mut [f32] {
		self.raw_buffer.borrow_mut()
	}
}

impl<'a, const N_CH: usize, Buffer: Borrow<[f32]>> IntoIterator
	for &'a InterleavedAudioBuffer<N_CH, Buffer>
{
	type IntoIter = InterleavedAudioBufferIter<'a, N_CH, Buffer>;
	type Item = AudioFrame<N_CH, &'a [f32]>;
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a, const N_CH: usize, Buffer: BorrowMut<[f32]>> IntoIterator
	for &'a mut InterleavedAudioBuffer<N_CH, Buffer>
{
	type IntoIter = InterleavedAudioBufferIterMut<'a, N_CH, Buffer>;
	type Item = AudioFrame<N_CH, &'a mut [f32]>;
	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

pub trait InterleavedAudioBufferBase: Send + Sync {
	fn at_boxed(&self, index: usize) -> Box<dyn AudioFrameTrait>;
	// fn iter(&self) -> InterleavedAudioBufferIterTrait;
	fn n_of_frames(&self) -> usize;
	fn raw_buffer(&self) -> &[f32];
}

impl<const N_CH: usize, Buffer: Borrow<[f32]> + Send + Sync> InterleavedAudioBufferBase
	for InterleavedAudioBuffer<N_CH, Buffer>
{
	fn at_boxed(&self, index: usize) -> Box<dyn AudioFrameTrait> {
		let frame = AudioFrame::<N_CH, _>::new(self.at(index).to_vec());
		Box::new(frame) as Box<dyn AudioFrameTrait>
	}

	fn n_of_frames(&self) -> usize {
		self.n_of_frames()
	}

	fn raw_buffer(&self) -> &[f32] {
		self.raw_buffer()
	}
}

impl<const N_CH: usize, Buffer: Borrow<[f32]>> ToMono for InterleavedAudioBuffer<N_CH, Buffer> {
	type Target = Vec<f32>;

	fn to_mono(&self) -> Self::Target {
		self.to_mono()
	}
}

impl<const N_CH: usize, Buffer: Borrow<[f32]>> NOfChannels
	for InterleavedAudioBuffer<N_CH, Buffer>
{
	fn n_of_channels(&self) -> usize {
		N_CH
	}
}

impl<const N_CH: usize, A: Borrow<[f32]>, B: Borrow<[f32]>>
	PartialEq<InterleavedAudioBuffer<N_CH, B>> for InterleavedAudioBuffer<N_CH, A>
{
	fn eq(&self, other: &InterleavedAudioBuffer<N_CH, B>) -> bool {
		self.raw_buffer.borrow() == other.raw_buffer.borrow()
	}
}

impl<const N_CH: usize, A: Borrow<[f32]>, B: Borrow<[f32]>>
	PartialOrd<InterleavedAudioBuffer<N_CH, B>> for InterleavedAudioBuffer<N_CH, A>
{
	fn partial_cmp(&self, other: &InterleavedAudioBuffer<N_CH, B>) -> Option<std::cmp::Ordering> {
		self.raw_buffer
			.borrow()
			.partial_cmp(other.raw_buffer.borrow())
	}
}

// impl<const N_CH: usize, Buffer: BorrowMut<[f32]>> Index<usize>
// 	for InterleavedAudioBuffer<N_CH, Buffer>
// {
// 	type Output = f32;

// 	fn index(&self, index: usize) -> &Self::Output {
// 		&self.raw_buffer.borrow()[index]
// 	}
// }

// impl<const N_CH: usize, Buffer: BorrowMut<[f32]>> IndexMut<usize>
// 	for InterleavedAudioBuffer<N_CH, Buffer>
// {
// 	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
// 		&mut self.raw_buffer.borrow_mut()[index]
// 	}
// }

impl<const N_CH: usize, Buffer: Borrow<[f32]>> Deref for InterleavedAudioBuffer<N_CH, Buffer> {
	type Target = [f32];

	fn deref(&self) -> &Self::Target {
		self.raw_buffer.borrow()
	}
}

impl<const N_CH: usize, Buffer: BorrowMut<[f32]>> DerefMut
	for InterleavedAudioBuffer<N_CH, Buffer>
{
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.raw_buffer.borrow_mut()
	}
}

pub trait InterleavedAudioBufferTrait:
	InterleavedAudioBufferBase
	+ ToMono<Target = Vec<f32>>
	+ NOfChannels
	/* + Index<usize> */
	+ Deref<Target = [f32]>
{
}

impl<const N_CH: usize, Buffer: Borrow<[f32]> + Send + Sync> InterleavedAudioBufferTrait
	for InterleavedAudioBuffer<N_CH, Buffer>
{
}

pub trait InterleavedAudioBufferTraitMut:
	InterleavedAudioBufferTrait /* + IndexMut<usize> */ + DerefMut<Target = [f32]>
{
}

impl<const N_CH: usize, Buffer: BorrowMut<[f32]> + Send + Sync> InterleavedAudioBufferTraitMut
	for InterleavedAudioBuffer<N_CH, Buffer>
{
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_snapshot_iterator() {
		let snapshot = InterleavedAudioBuffer::<2, _>::new(&[1., 2., 3., 4., 5., 6., 7., 8.][..]);
		let mut iter = snapshot.iter();

		assert_eq!(
			iter.next(),
			Some(AudioFrame::new(&[1.0f32, 2.0f32] as &[f32]))
		);
		assert_eq!(
			iter.next(),
			Some(AudioFrame::new(&[3.0f32, 4.0f32] as &[f32]))
		);
		assert_eq!(
			iter.next(),
			Some(AudioFrame::new(&[5.0f32, 6.0f32] as &[f32]))
		);
		assert_eq!(
			iter.next(),
			Some(AudioFrame::new(&[7.0f32, 8.0f32] as &[f32]))
		);
		assert_eq!(iter.next(), None);
	}
	#[test]
	fn test_snapshot_indexing() {
		let snapshot = InterleavedAudioBuffer::<2, _>::new([1., 2., 3., 4., 5., 6., 7., 8.]);
		assert_eq!(snapshot.at(0), AudioFrame::new([1., 2.]));
		assert_eq!(snapshot.at(1), AudioFrame::new([3., 4.]));
		assert_eq!(snapshot.at(2), AudioFrame::new([5., 6.]));
		assert_eq!(snapshot.at(3), AudioFrame::new([7., 8.]));
	}
	#[test]
	fn test_from_mono() {
		let snapshot = InterleavedAudioBuffer::<1, _>::new([1., 2., 3., 4., 5., 6., 7., 8.]);
		assert_eq!(snapshot.at(0), AudioFrame::new([1.]));
		assert_eq!(snapshot.at(1), AudioFrame::new([2.]));
		assert_eq!(snapshot.at(2), AudioFrame::new([3.]));
		assert_eq!(snapshot.at(3), AudioFrame::new([4.]));
		assert_eq!(snapshot.at(4), AudioFrame::new([5.]));
		assert_eq!(snapshot.at(5), AudioFrame::new([6.]));
		assert_eq!(snapshot.at(6), AudioFrame::new([7.]));
		assert_eq!(snapshot.at(7), AudioFrame::new([8.]));
	}
}

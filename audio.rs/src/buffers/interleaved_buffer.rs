use std::borrow::{Borrow, BorrowMut};

use crate::{NOfFrames, SampleRate, SamplingCtx};

use super::{frame_buffer::AudioFrame, InterleavedAudioBufferIter, InterleavedAudioBufferIterMut};

// TODO: get_channel() -> Vec<f32>

#[derive(Debug)]
pub struct InterleavedAudioBuffer<Buffer: Borrow<[f32]>> {
	sampling_ctx: SamplingCtx,
	raw_buffer: Buffer,
}

impl<Buffer: Borrow<[f32]>> InterleavedAudioBuffer<Buffer> {
	/// Creates a new [`InterleavedAudioBuffer`].
	///
	/// # Panics
	/// - if the buffer size is not a multiple of the number of channels.
	#[must_use]
	pub fn new(sampling_ctx: SamplingCtx, raw_buffer: Buffer) -> Self {
		assert_eq!(
			raw_buffer.borrow().len() % sampling_ctx.n_ch(),
			0,
			"buffer size must be a multiple of the number of channels"
		);
		Self {
			sampling_ctx,
			raw_buffer,
		}
	}

	#[must_use]
	#[allow(clippy::missing_panics_doc)] // REASON: invariant guaranteed by `assert_eq` in the constructor
	pub fn at(&self, index: usize) -> AudioFrame<&[f32]> {
		AudioFrame::new(
			self.raw_buffer.borrow()[index * self.n_ch()..(index + 1) * self.n_ch()]
				.try_into()
				.unwrap(),
		)
	}

	/// # Panics
	/// - if the two signals are incompatible (different number of channels or different sample rate)
	#[must_use]
	pub fn concat(&self, other: &Self) -> InterleavedAudioBuffer<Vec<f32>> {
		assert_eq!(self.n_ch(), other.n_ch());
		assert_eq!(self.sample_rate(), other.sample_rate());
		InterleavedAudioBuffer::new(self.sampling_ctx, {
			let mut base = self.raw_buffer.borrow().to_vec();
			base.extend(other.raw_buffer.borrow());
			base
		})
	}

	#[must_use]
	pub fn iter(&self) -> InterleavedAudioBufferIter<Buffer> {
		InterleavedAudioBufferIter::new(self)
	}

	#[must_use]
	pub fn into_raw(self) -> (SamplingCtx, Buffer) {
		(self.sampling_ctx, self.raw_buffer)
	}

	/// The number of frames corresponds to the number of sampling points in time, regardless of the number
	/// of channels.
	#[must_use]
	pub fn n_of_frames(&self) -> NOfFrames {
		self.sampling_ctx
			.samples_to_frames(self.raw_buffer.borrow().len())
	}

	#[must_use]
	pub fn sampling_ctx(&self) -> SamplingCtx {
		self.sampling_ctx
	}

	#[must_use]
	pub fn sample_rate(&self) -> SampleRate {
		self.sampling_ctx.sample_rate()
	}

	/// Converts this interleaved collection to a raw buffer containing the samples of a mono track.
	/// Samples in the mono track are the average of all the channel samples for each point in time.
	#[must_use]
	pub fn to_mono(&self) -> Vec<f32> {
		if self.n_ch() == 1 {
			self.raw_buffer.borrow().to_vec()
		} else {
			self.iter().map(|frame| frame.to_mono()).collect()
		}
	}

	#[must_use]
	pub fn n_ch(&self) -> usize {
		self.sampling_ctx.n_ch()
	}

	#[must_use]
	pub fn raw_buffer(&self) -> &Buffer {
		&self.raw_buffer
	}

	#[must_use]
	pub fn cloned(&self) -> InterleavedAudioBuffer<Vec<f32>> {
		InterleavedAudioBuffer::new(self.sampling_ctx, self.raw_buffer.borrow().to_vec())
	}
}

impl<Buffer: BorrowMut<[f32]>> InterleavedAudioBuffer<Buffer> {
	/// # Panics
	/// - if the index is out of bound.
	#[must_use]
	pub fn at_mut(&mut self, index: usize) -> AudioFrame<&mut [f32]> {
		assert!(index < self.n_of_frames().0);

		let n_ch = self.n_ch();

		AudioFrame::new(&mut self.raw_buffer.borrow_mut()[index * n_ch..(index + 1) * n_ch])
	}

	#[must_use]
	pub fn iter_mut(&mut self) -> InterleavedAudioBufferIterMut<Buffer> {
		InterleavedAudioBufferIterMut::new(self)
	}

	#[must_use]
	pub fn raw_buffer_mut(&mut self) -> &mut Buffer {
		&mut self.raw_buffer
	}
}

impl<BufferA: Borrow<[f32]>, BufferB: Borrow<[f32]>> PartialEq<InterleavedAudioBuffer<BufferB>>
	for InterleavedAudioBuffer<BufferA>
{
	fn eq(&self, other: &InterleavedAudioBuffer<BufferB>) -> bool {
		self.sampling_ctx == other.sampling_ctx
			&& self.raw_buffer.borrow() == other.raw_buffer.borrow()
	}
}

impl<'a, Buffer: Borrow<[f32]>> IntoIterator for &'a InterleavedAudioBuffer<Buffer> {
	type Item = AudioFrame<&'a [f32]>;
	type IntoIter = InterleavedAudioBufferIter<'a, Buffer>;
	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a, Buffer: BorrowMut<[f32]>> IntoIterator for &'a mut InterleavedAudioBuffer<Buffer> {
	type Item = AudioFrame<&'a mut [f32]>;
	type IntoIter = InterleavedAudioBufferIterMut<'a, Buffer>;
	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

impl<FrameBuffer: Borrow<[f32]>> Extend<AudioFrame<FrameBuffer>>
	for InterleavedAudioBuffer<Vec<f32>>
{
	fn extend<T: IntoIterator<Item = AudioFrame<FrameBuffer>>>(&mut self, iter: T) {
		self.raw_buffer
			.extend(iter.into_iter().flat_map(|frame| frame.samples().to_vec()));
	}
}

impl<Buffer: Borrow<[f32]>> AsRef<[f32]> for InterleavedAudioBuffer<Buffer> {
	fn as_ref(&self) -> &[f32] {
		self.raw_buffer.borrow()
	}
}

impl<Buffer: BorrowMut<[f32]>> AsMut<[f32]> for InterleavedAudioBuffer<Buffer> {
	fn as_mut(&mut self) -> &mut [f32] {
		self.raw_buffer.borrow_mut()
	}
}

#[cfg(test)]
mod tests {
	use std::time::Duration;

	use super::*;

	#[test]
	fn test_snapshot_iterator() {
		let sampling_ctx = SamplingCtx::new(44100.into(), 2);
		let snapshot =
			InterleavedAudioBuffer::new(sampling_ctx, [1., 2., 3., 4., 5., 6., 7., 8.].as_slice());
		let mut iter = snapshot.iter();

		assert_eq!(
			iter.next(),
			Some(AudioFrame::new([1.0f32, 2.0f32].as_slice()))
		);
		assert_eq!(
			iter.next(),
			Some(AudioFrame::new([3.0f32, 4.0f32].as_slice()))
		);
		assert_eq!(
			iter.next(),
			Some(AudioFrame::new([5.0f32, 6.0f32].as_slice()))
		);
		assert_eq!(
			iter.next(),
			Some(AudioFrame::new([7.0f32, 8.0f32].as_slice()))
		);
		assert_eq!(iter.next(), None);
	}

	#[test]
	fn test_snapshot_mut_iterator() {
		let sampling_ctx = SamplingCtx::new(44100.into(), 2);
		let mut raw_buffer = [1., 2., 3., 4., 5., 6., 7., 8.];
		let mut snapshot = InterleavedAudioBuffer::new(sampling_ctx, raw_buffer.as_mut_slice());

		for mut frame in &mut snapshot {
			frame[0] -= 10.;
			frame[1] += 10.;
		}
		let mut iter = snapshot.iter();
		assert_eq!(
			iter.next(),
			Some(AudioFrame::new([-10. + 1.0f32, 10. + 2.0f32].as_slice()))
		);
		assert_eq!(
			iter.next(),
			Some(AudioFrame::new([-10. + 3.0f32, 10. + 4.0f32].as_slice()))
		);
		assert_eq!(
			iter.next(),
			Some(AudioFrame::new([-10. + 5.0f32, 10. + 6.0f32].as_slice()))
		);
		assert_eq!(
			iter.next(),
			Some(AudioFrame::new([-10. + 7.0f32, 10. + 8.0f32].as_slice()))
		);
		assert_eq!(iter.next(), None);
	}

	#[test]
	fn test_snapshot_indexing() {
		let snapshot = InterleavedAudioBuffer::new(
			SamplingCtx::new(SampleRate(44100), 2),
			[1., 2., 3., 4., 5., 6., 7., 8.],
		);
		assert_eq!(snapshot.at(0), AudioFrame::new([1., 2.]));
		assert_eq!(snapshot.at(1), AudioFrame::new([3., 4.]));
		assert_eq!(snapshot.at(2), AudioFrame::new([5., 6.]));
		assert_eq!(snapshot.at(3), AudioFrame::new([7., 8.]));
	}
	#[test]
	fn test_from_mono() {
		let snapshot = InterleavedAudioBuffer::new(
			SamplingCtx::new(SampleRate(44100), 1),
			[1., 2., 3., 4., 5., 6., 7., 8.],
		);
		assert_eq!(snapshot.at(0), AudioFrame::new([1.]));
		assert_eq!(snapshot.at(1), AudioFrame::new([2.]));
		assert_eq!(snapshot.at(2), AudioFrame::new([3.]));
		assert_eq!(snapshot.at(3), AudioFrame::new([4.]));
		assert_eq!(snapshot.at(4), AudioFrame::new([5.]));
		assert_eq!(snapshot.at(5), AudioFrame::new([6.]));
		assert_eq!(snapshot.at(6), AudioFrame::new([7.]));
		assert_eq!(snapshot.at(7), AudioFrame::new([8.]));
	}
	#[test]
	fn test_duration() {
		let sampling_ctx = SamplingCtx::new(SampleRate(44100), 1);
		let snapshot = InterleavedAudioBuffer::new(sampling_ctx, vec![0.; 4410]);
		assert_eq!(
			sampling_ctx.frames_to_duration(snapshot.n_of_frames()),
			Duration::from_millis(100)
		);
	}
}

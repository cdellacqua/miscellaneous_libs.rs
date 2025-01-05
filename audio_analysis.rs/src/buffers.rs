use std::ops::{Index, IndexMut};

pub struct InterleavedAudioSamples {
	pub buffer: Vec<f32>,
	pub n_of_channels: usize,
}

impl InterleavedAudioSamples {
	#[must_use]
	pub fn new(buffer: Vec<f32>, n_of_channels: usize) -> Self {
		Self {
			buffer,
			n_of_channels,
		}
	}

	#[must_use]
	pub fn iter(&self) -> InterleavedAudioSamplesIter {
		InterleavedAudioSamplesIter::new(&self.buffer, self.n_of_channels)
	}

	///
	/// The number of frames corresponds to the number of sampling points in time, regardless of the number
	/// of channels.
	///
	#[must_use]
	pub fn n_of_frames(&self) -> usize {
		self.buffer.len() / self.n_of_channels
	}

	///
	/// Converts this interleaved collection to a raw buffer containing the samples of a mono track.
	/// Samples in the mono track are the average of all the channel samples for each point in time.
	///
	#[must_use]
	pub fn to_mono(&self) -> Vec<f32> {
		self.iter()
			.map(|channels| {
				#[allow(clippy::cast_precision_loss)]
				return channels.iter().sum::<f32>() / self.n_of_channels as f32;
			})
			.collect()
	}
}

pub struct InterleavedAudioSamplesIter<'a> {
	i: usize,
	max: usize,
	buffer: &'a [f32],
	n_of_channels: usize,
}

impl<'a> InterleavedAudioSamplesIter<'a> {
	fn new(buffer: &'a [f32], n_of_channels: usize) -> Self {
		Self {
			i: 0,
			max: buffer.len() / n_of_channels,
			buffer,
			n_of_channels,
		}
	}
}

impl<'a> Iterator for InterleavedAudioSamplesIter<'a> {
	type Item = &'a [f32];

	fn next(&mut self) -> Option<Self::Item> {
		if self.i < self.max {
			let slice =
				&self.buffer[self.i * self.n_of_channels..(self.i + 1) * self.n_of_channels];

			self.i += 1;

			Some(slice)
		} else {
			None
		}
	}
}

impl Index<usize> for InterleavedAudioSamples {
	type Output = [f32];

	fn index(&self, index: usize) -> &Self::Output {
		&self.buffer[index * self.n_of_channels..(index + 1) * self.n_of_channels]
	}
}

impl IndexMut<usize> for InterleavedAudioSamples {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.buffer[index * self.n_of_channels..(index + 1) * self.n_of_channels]
	}
}

impl<'a> IntoIterator for &'a InterleavedAudioSamples {
	type Item = &'a [f32];

	type IntoIter = InterleavedAudioSamplesIter<'a>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_snapshot_iterator() {
		let snapshot = InterleavedAudioSamples::new(vec![1., 2., 3., 4., 5., 6., 7., 8.], 2);
		let iter = &mut snapshot.into_iter();
		assert_eq!(iter.next(), Some(&[1.0f32, 2.0f32] as &[f32]));
		assert_eq!(iter.next(), Some(&[3.0f32, 4.0f32] as &[f32]));
		assert_eq!(iter.next(), Some(&[5.0f32, 6.0f32] as &[f32]));
		assert_eq!(iter.next(), Some(&[7.0f32, 8.0f32] as &[f32]));
		assert_eq!(iter.next(), None);
	}
	#[test]
	fn test_snapshot_indexing() {
		let snapshot = InterleavedAudioSamples::new(vec![1., 2., 3., 4., 5., 6., 7., 8.], 2);
		assert_eq!(snapshot[0], [1., 2.]);
		assert_eq!(snapshot[1], [3., 4.]);
		assert_eq!(snapshot[2], [5., 6.]);
		assert_eq!(snapshot[3], [7., 8.]);
	}
}

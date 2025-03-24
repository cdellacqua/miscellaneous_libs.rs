use std::borrow::Borrow;

pub struct BufferHopper<T> {
	buffer: Vec<T>,
	batch_size: usize,
	processed_batches: usize,
}

impl<T: Copy> BufferHopper<T> {
	#[must_use]
	pub fn new(batch_size: usize) -> Self {
		Self {
			buffer: Vec::with_capacity(batch_size),
			batch_size,
			processed_batches: 0,
		}
	}

	// Desperately awaiting for (sync) generators to get stabilized
	pub fn feed<Data: Borrow<[T]>, Processor: FnMut(&mut [T], usize /* batch_idx */)>(
		&mut self,
		data: Data,
		mut processor: Processor,
	) {
		let mut i = 0;
		let data = data.borrow();
		while i < data.len() {
			let fillable = (self.batch_size - self.buffer.len()).min(data.len() - i);
			self.buffer.extend_from_slice(&data[i..i + fillable]);

			debug_assert!(self.buffer.len() <= self.batch_size);

			i += fillable;

			if self.buffer.len() == self.batch_size {
				processor(&mut self.buffer, self.processed_batches);
				self.processed_batches += 1;
				self.buffer.clear();
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::BufferHopper;

	#[test]
	fn test_batch_idx_matches_n_of_calls() {
		let mut calls = 0;
		let mut hopper = BufferHopper::new(2);

		for data in [[0, 1].as_slice(), &[2, 3], &[4], &[5]] {
			hopper.feed(data, |_, idx| {
				assert_eq!(calls, idx);
				calls += 1;
			});
		}
		assert_eq!(calls, 3);
	}

	#[test]
	fn test_exact() {
		let mut hopper = BufferHopper::new(2);
		for data in [[0, 1], [2, 3]] {
			hopper.feed(data, |batch, idx| {
				assert_eq!(&[idx * 2, idx * 2 + 1], batch);
			});
		}
	}

	#[test]
	fn test_one_at_a_time() {
		let mut processor = BufferHopper::new(2);
		for i in 0..10 {
			processor.feed([i], |batch, idx| {
				assert_eq!(&[idx * 2, idx * 2 + 1], batch);
			});
		}
	}

	#[test]
	fn test_general_use_case() {
		let mut last_batch = [0; 3];
		let mut processor = BufferHopper::new(3);

		for data in [
			[0, 1].as_slice(),
			&[2, 3, 4, 5],
			&[6],
			&[7],
			&[8, 9],
			&[10, 11, 12, 13, 14, 15],
			&[16, 17, 18, 19],
			&[20, 21, 22],
			&[23, 24],
			&[25],
			&[],
			&[],
			&[],
			&[26],
		] {
			processor.feed(data, |batch, idx| {
				assert_eq!(&[idx * 3, idx * 3 + 1, idx * 3 + 2], batch);
				last_batch.copy_from_slice(batch);
			});
		}
		assert_eq!(last_batch, [24, 25, 26]);
	}
}

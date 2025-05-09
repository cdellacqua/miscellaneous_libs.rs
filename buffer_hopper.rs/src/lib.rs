use std::borrow::Borrow;

pub struct BufferHopper<T> {
	buffer: Vec<T>,
	batch_size: usize,
	overlap: usize,
	processed_batches: usize,
}

impl<T: Copy> BufferHopper<T> {
	#[must_use]
	pub fn new(batch_size: usize) -> Self {
		debug_assert!(batch_size > 0, "batch_size must be greater than 0");
		Self {
			buffer: Vec::with_capacity(batch_size),
			batch_size,
			overlap: 0,
			processed_batches: 0,
		}
	}

	#[must_use]
	pub fn new_with_overlap(batch_size: usize, overlap: usize) -> Self {
		debug_assert!(
			batch_size > overlap,
			"batch_size ({batch_size}) must be greater than the overlap ({overlap})"
		);
		Self {
			buffer: Vec::with_capacity(batch_size),
			batch_size,
			overlap,
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
				self.buffer
					.copy_within(self.batch_size - self.overlap..self.batch_size, 0);
				self.buffer.truncate(self.overlap);
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

	#[test]
	fn test_batch_with_overlap() {
		let mut calls = 0;
		let mut hopper = BufferHopper::new_with_overlap(4, 3);

		for data in [
			[0, 1].as_slice(),
			&[2, 3],
			&[4, 5, 6, 7, 8, 9],
			&[10, 11, 12],
		] {
			hopper.feed(data, |batch, idx| {
				assert_eq!(calls, idx);

				match calls {
					0 => assert_eq!(batch, [0, 1, 2, 3]),
					1 => assert_eq!(batch, [1, 2, 3, 4]),
					2 => assert_eq!(batch, [2, 3, 4, 5]),
					3 => assert_eq!(batch, [3, 4, 5, 6]),
					4 => assert_eq!(batch, [4, 5, 6, 7]),
					5 => assert_eq!(batch, [5, 6, 7, 8]),
					6 => assert_eq!(batch, [6, 7, 8, 9]),
					7 => assert_eq!(batch, [7, 8, 9, 10]),
					8 => assert_eq!(batch, [8, 9, 10, 11]),
					9 => assert_eq!(batch, [9, 10, 11, 12]),
					_ => unreachable!(),
				}
				calls += 1;
			});
		}
		assert_eq!(calls, 10);
	}

	#[test]
	fn test_batch_with_overlap_of_one() {
		let mut calls = 0;
		let mut hopper = BufferHopper::new_with_overlap(4, 1);

		for data in [
			[0, 1].as_slice(),
			&[2, 3],
			&[4, 5, 6, 7, 8, 9],
			&[10, 11, 12],
		] {
			hopper.feed(data, |batch, idx| {
				assert_eq!(calls, idx);

				match calls {
					0 => assert_eq!(batch, [0, 1, 2, 3]),
					1 => assert_eq!(batch, [3, 4, 5, 6]),
					2 => assert_eq!(batch, [6, 7, 8, 9]),
					3 => assert_eq!(batch, [9, 10, 11, 12]),
					_ => unreachable!(),
				}
				calls += 1;
			});
		}
		assert_eq!(calls, 4);
	}

	#[cfg(debug_assertions)]
	#[test]
	#[should_panic(expected = "batch_size (4) must be greater than the overlap (5)")]
	fn test_batch_with_overlap_greater_than_batch_size() {
		let _hopper = BufferHopper::<i32>::new_with_overlap(4, 5);
	}

	#[cfg(debug_assertions)]
	#[test]
	#[should_panic(expected = "batch_size must be greater than 0")]
	fn test_batch_with_batch_size_of_zero() {
		let _hopper = BufferHopper::<i32>::new(0);
	}
}

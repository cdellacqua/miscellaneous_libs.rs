use std::borrow::Borrow;

pub struct BufferHopper<
	T,
	const N: usize,
	Processor: FnMut(&mut [T; N], usize /* batch_idx */),
> {
	batch: [T; N],
	batch_i: usize,
	processed_batches: usize,
	processor: Processor,
}

impl<T: Copy + Default, const N: usize, Processor: FnMut(&mut [T; N], usize /* batch_idx */)>
	BufferHopper<T, N, Processor>
{
	#[must_use]
	pub fn new(processor: Processor) -> Self {
		Self {
			batch: [T::default(); N],
			batch_i: 0,
			processed_batches: 0,
			processor,
		}
	}

	pub fn feed<Data: Borrow<[T]>>(&mut self, data: Data) {
		let mut i = 0;
		let data = data.borrow();
		while i < data.len() {
			let fillable = (N - self.batch_i).min(data.len() - i);
			self.batch[self.batch_i..self.batch_i + fillable]
				.copy_from_slice(&data[i..i + fillable]);
			self.batch_i += fillable;
			i += fillable;

			if self.batch_i == N {
				(self.processor)(&mut self.batch, self.processed_batches);
				self.processed_batches += 1;
				self.batch_i = 0;
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
		let mut processor = BufferHopper::<_, 2, _>::new(|_, idx| {
			assert_eq!(calls, idx);
			calls += 1;
		});
		processor.feed([0, 1].as_slice());
		processor.feed([2, 3]);
		processor.feed([4]);
		processor.feed([5]);
		assert_eq!(calls, 3);
	}

	#[test]
	fn test_exact() {
		let mut processor = BufferHopper::new(|batch, idx| {
			assert_eq!(&[idx * 2, idx * 2 + 1], batch);
		});
		processor.feed([0, 1]);
		processor.feed([2, 3]);
	}

	#[test]
	fn test_one_at_a_time() {
		let mut processor = BufferHopper::new(|batch, idx| {
			assert_eq!(&[idx * 2, idx * 2 + 1], batch);
		});
		for i in 0..10 {
			processor.feed([i]);
		}
	}

	#[test]
	fn test_general_use_case() {
		let mut last_batch = [0; 3];
		let mut processor = BufferHopper::new(|batch, idx| {
			assert_eq!(&[idx * 3, idx * 3 + 1, idx * 3 + 2], batch);
			last_batch = *batch;
		});
		processor.feed([0, 1]);
		processor.feed([2, 3, 4, 5]);
		processor.feed([6]);
		processor.feed([7]);
		processor.feed([8, 9]);
		processor.feed([10, 11, 12, 13, 14, 15]);
		processor.feed([16, 17, 18, 19]);
		processor.feed([20, 21, 22]);
		processor.feed([23, 24]);
		processor.feed([25]);
		processor.feed([]);
		processor.feed([]);
		processor.feed([]);
		processor.feed([26]);
		assert_eq!(last_batch, [24, 25, 26]);
	}
}

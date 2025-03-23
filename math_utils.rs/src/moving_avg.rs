use std::ops::Add;

use ringbuffer::{AllocRingBuffer, RingBuffer};

use crate::ext::DivisibleByUsize;

#[derive(Debug, Clone)]
pub struct MovingAverage<T> {
	series: AllocRingBuffer<T>,
}

impl<T: Add<T, Output = T> + DivisibleByUsize + Default + Copy> MovingAverage<T> {
	#[must_use]
	pub fn new(window_size: usize) -> Self {
		Self {
			series: AllocRingBuffer::new(window_size),
		}
	}

	pub fn push(&mut self, value: T) {
		self.series.push(value);
	}

	#[must_use]
	pub fn is_window_full(&self) -> bool {
		self.series.is_full()
	}

	#[must_use]
	pub fn is_window_empty(&self) -> bool {
		self.series.is_empty()
	}

	pub fn reset(&mut self) {
		self.series.clear();
	}

	#[allow(clippy::missing_panics_doc)] // REASON: invariant guaranteed by explicit check at the beginning of the function
	#[must_use]
	pub fn avg(&self) -> T {
		if self.series.is_empty() {
			T::default()
		} else {
			let sum = self
				.series
				.iter()
				.copied()
				.reduce(|acc, cur| acc + cur)
				.expect("internal error: at least one element in the series");
			sum.div_usize(self.series.len())
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_moving_avg() {
		let mut avg = MovingAverage::<f32>::new(3);
		assert!(avg.avg() < f32::EPSILON);
		avg.push(0.0);
		assert!(avg.avg() < f32::EPSILON);
		avg.push(1.0);
		assert!((avg.avg() - 0.5).abs() < f32::EPSILON);
		avg.push(1.0);
		assert!((avg.avg() - 0.67).abs() < 0.01);
		avg.push(1.0);
		assert!((avg.avg() - 1.).abs() < f32::EPSILON);
		avg.push(2.0);
		assert!((avg.avg() - 1.33).abs() < 0.01);
	}
}

use hashbrown::{HashMap, HashSet};

use crate::{even_odd::IsEven, ext::DivisibleByUsize};
use std::{
	borrow::{Borrow, BorrowMut},
	cell::RefCell,
	cmp::Ordering,
	hash::Hash,
	ops::{Add, Mul, Sub},
};

#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatisticsError {
	#[error("common stats are undefined on empty series")]
	EmptySeries,
}

#[derive(Debug, Clone)]
pub struct SeriesStatistics<T, Series: Borrow<[T]>> {
	series: Series,
	sum: RefCell<Option<T>>,
	mean: RefCell<Option<T>>,
	variance: RefCell<Option<T>>,
	max: RefCell<Option<T>>,
	min: RefCell<Option<T>>,
	median: RefCell<Option<T>>,
	mode: RefCell<Option<HashSet<T>>>,
}
impl<T, Series: Borrow<[T]>> SeriesStatistics<T, Series> {
	/// # Errors
	/// - on empty series
	pub fn new(series: Series) -> Result<Self, StatisticsError> {
		if series.borrow().is_empty() {
			Err(StatisticsError::EmptySeries)
		} else {
			Ok(Self {
				series,
				sum: RefCell::default(),
				mean: RefCell::default(),
				variance: RefCell::default(),
				max: RefCell::default(),
				min: RefCell::default(),
				median: RefCell::default(),
				mode: RefCell::default(),
			})
		}
	}

	pub fn series(&self) -> &[T] {
		self.series.borrow()
	}
}

impl<T, Series: BorrowMut<[T]>> SeriesStatistics<T, Series> {
	pub fn series_mut(&mut self) -> &mut [T] {
		self.series.borrow_mut()
	}
}

impl<T: Add<T, Output = T> + Copy, Series: Borrow<[T]>> SeriesStatistics<T, Series> {
	#[allow(clippy::missing_panics_doc)] // REASON: invariant (series.len() > 0) guaranteed by explicit check in the constructor
	#[must_use]
	pub fn sum(&self) -> T {
		*self.sum.borrow_mut().get_or_insert_with(|| {
			let series = self.series.borrow();
			series
				.iter()
				.copied()
				.reduce(|acc, cur| acc + cur)
				.expect("internal error: at least one element should be present in the series")
		})
	}
}

impl<T: Add<T, Output = T> + DivisibleByUsize + Copy, Series: Borrow<[T]>>
	SeriesStatistics<T, Series>
{
	#[must_use]
	pub fn mean(&self) -> T {
		let sum = self.sum();
		*self.mean.borrow_mut().get_or_insert_with(|| {
			let series = self.series.borrow();
			sum.div_usize(series.len())
		})
	}
}

impl<T: PartialOrd + Copy, Series: Borrow<[T]>> SeriesStatistics<T, Series> {
	#[allow(clippy::missing_panics_doc)] // REASON: invariant (series.len() > 0) guaranteed by explicit check in the constructor
	#[must_use]
	pub fn max(&self) -> T {
		*self.max.borrow_mut().get_or_insert_with(|| {
			let series = self.series.borrow();
			*series
				.iter()
				.max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
				.expect("internal error: at least one element should be present in the series")
		})
	}

	#[allow(clippy::missing_panics_doc)] // REASON: invariant (series.len() > 0) guaranteed by explicit check in the constructor
	#[must_use]
	pub fn min(&self) -> T {
		*self.min.borrow_mut().get_or_insert_with(|| {
			let series = self.series.borrow();
			*series
				.iter()
				.min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
				.expect("internal error: at least one element should be present in the series")
		})
	}
}

impl<T: PartialOrd + Add<T, Output = T> + DivisibleByUsize + Copy, Series: Borrow<[T]>>
	SeriesStatistics<T, Series>
{
	#[must_use]
	pub fn mid_range(&self) -> T {
		(self.max() + self.min()).div_usize(2)
	}

	#[must_use]
	pub fn median(&self) -> T {
		*self.median.borrow_mut().get_or_insert_with(|| {
			let borrow: &[T] = self.series.borrow();
			let mut series = borrow.to_vec();
			series.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
			let len = series.len();
			if len.is_even() {
				(series[len / 2 - 1] + series[len / 2]).div_usize(2)
			} else {
				series[len / 2]
			}
		})
	}
}

impl<T: Hash + Eq + Copy, Series: Borrow<[T]>> SeriesStatistics<T, Series> {
	#[allow(clippy::missing_panics_doc)] // REASON: invariant (series.len() > 0) guaranteed by explicit check in the constructor
	#[must_use]
	pub fn mode(&self) -> HashSet<T> {
		self.mode
			.borrow_mut()
			.get_or_insert_with(|| {
				let borrow: &[T] = self.series.borrow();
				let mut frequencies = HashMap::new();
				for item in borrow {
					frequencies
						.entry(item)
						.and_modify(|n| *n += 1)
						.or_insert(0usize);
				}
				let mut frequencies_vec: Vec<(&T, usize)> = frequencies.into_iter().collect();
				frequencies_vec.sort_by(|(_, a), (_, b)| a.cmp(b).reverse());
				let (_, max_count) = frequencies_vec[0];

				frequencies_vec
					.into_iter()
					.map_while(|(item, count)| {
						if max_count == count {
							Some(*item)
						} else {
							None
						}
					})
					.collect()
			})
			.clone()
	}
}

impl<
		T: Add<T, Output = T> + Sub<T, Output = T> + DivisibleByUsize + Mul<T, Output = T> + Copy,
		Series: Borrow<[T]>,
	> SeriesStatistics<T, Series>
{
	#[allow(clippy::missing_panics_doc)] // REASON: invariant (series.len() > 0) guaranteed by explicit check in the constructor
	#[must_use]
	pub fn variance(&self) -> T {
		*self.variance.borrow_mut().get_or_insert_with(|| {
			let series = self.series.borrow();
			self.series
				.borrow()
				.iter()
				.map(|v| {
					let diff = self.mean() - *v;
					diff * diff
				})
				.reduce(|acc, cur| acc + cur)
				.expect("internal error: at least one element should be present in the series")
				.div_usize(series.len())
		})
	}
}

impl<Series: Borrow<[f32]>> SeriesStatistics<f32, Series> {
	pub fn std_dev(&self) -> f32 {
		self.variance().sqrt()
	}
}

impl<Series: Borrow<[f64]>> SeriesStatistics<f64, Series> {
	pub fn std_dev(&self) -> f64 {
		self.variance().sqrt()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_standard_deviation() {
		let values: &[f32] = &[2., 4., 4., 4., 5., 5., 7., 9.];
		let stats = SeriesStatistics::new(values).unwrap();
		assert!((stats.variance().sqrt() - 2.).abs() < f32::EPSILON);
		assert!((stats.std_dev() - 2.).abs() < f32::EPSILON);
	}

	#[test]
	fn test_mean() {
		let values: &[f64] = &[1., 3., 3., 6., 7., 8., 9.];
		let stats = SeriesStatistics::new(values).unwrap();
		assert!((stats.mean() - 5.28).abs() < 0.01);
	}

	#[test]
	fn test_median() {
		let values: &[f64] = &[1., 3., 3., 6., 7., 8., 9.];
		let stats = SeriesStatistics::new(values).unwrap();
		assert!((stats.median() - 6.).abs() < f64::EPSILON);
	}

	#[test]
	fn test_median_even_series() {
		let values: &[f64] = &[1., 2., 3., 4., 5., 6., 8., 9.];
		let stats = SeriesStatistics::new(values).unwrap();
		assert!((stats.median() - 4.5).abs() < f64::EPSILON);
	}

	#[test]
	fn test_median_unsorted_series() {
		let values: &[f64] = &[4., 9., 5., 1., 3., 6., 8., 2.];
		let stats = SeriesStatistics::new(values).unwrap();
		assert!((stats.median() - 4.5).abs() < f64::EPSILON);
	}

	#[test]
	fn test_min() {
		let values: &[f64] = &[1., 2., 3., 4., 5., 6., 8., 9.];
		let stats = SeriesStatistics::new(values).unwrap();
		assert!((stats.min() - 1.).abs() < f64::EPSILON);
	}

	#[test]
	fn test_max() {
		let values: &[f64] = &[1., 2., 3., 4., 5., 6., 8., 9.];
		let stats = SeriesStatistics::new(values).unwrap();
		assert!((stats.max() - 9.).abs() < f64::EPSILON);
	}

	#[test]
	fn test_mode_single_value() {
		let values: &[i32] = &[1];
		let stats = SeriesStatistics::new(values).unwrap();
		assert_eq!(stats.mode(), HashSet::from_iter([1]));
	}

	#[test]
	fn test_mode() {
		let values: &[i32] = &[
			1, 1, 2, 3, 4, 5, 6, 8, 9, 8, 8, 8, 3, 3, 2, 3, 1, 3, 4, 3, 1,
		];
		let stats = SeriesStatistics::new(values).unwrap();
		assert_eq!(stats.mode(), HashSet::from_iter([3]));
	}

	#[test]
	fn test_bimodal() {
		let values: &[i32] = &[
			1, 1, 2, 3, 4, 5, 6, 8, 9, 8, 8, 8, 3, 3, 2, 3, 1, 3, 4, 3, 1, 1, 1,
		];
		let stats = SeriesStatistics::new(values).unwrap();
		assert_eq!(stats.mode(), HashSet::from_iter([1, 3,]));
	}
}

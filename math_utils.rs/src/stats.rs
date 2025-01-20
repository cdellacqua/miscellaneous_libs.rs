use crate::even_odd::IsEven;
use std::{borrow::Borrow, cell::RefCell};

#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatisticsError {
	#[error("common stats are undefined on empty series")]
	EmptySeries,
}

#[derive(Debug, Clone)]
pub struct SeriesStatistics<T, Series: Borrow<[T]>> {
	series: Series,
	mean: RefCell<Option<T>>,
	variance: RefCell<Option<T>>,
	max: RefCell<Option<T>>,
	min: RefCell<Option<T>>,
}

// Two distinct new methods because of the lack of overloading,
// although there probably is a cleaner way

impl<Series: Borrow<[f32]>> SeriesStatistics<f32, Series> {
	/// # Errors
	/// - on empty series
	pub fn new_f32(series: Series) -> Result<Self, StatisticsError> {
		if series.borrow().is_empty() {
			Err(StatisticsError::EmptySeries)
		} else {
			Ok(Self {
				series,
				mean: RefCell::default(),
				variance: RefCell::default(),
				max: RefCell::default(),
				min: RefCell::default(),
			})
		}
	}
}

impl<Series: Borrow<[f64]>> SeriesStatistics<f64, Series> {
	/// # Errors
	/// - on empty series
	pub fn new_f64(series: Series) -> Result<Self, StatisticsError> {
		if series.borrow().is_empty() {
			Err(StatisticsError::EmptySeries)
		} else {
			Ok(Self {
				series,
				mean: RefCell::default(),
				variance: RefCell::default(),
				max: RefCell::default(),
				min: RefCell::default(),
			})
		}
	}
}

macro_rules! impl_statistics_for {
	($t:ty) => {
		#[allow(clippy::cast_precision_loss)]
		impl<Series: Borrow<[$t]>> SeriesStatistics<$t, Series> {
			#[must_use]
			pub fn mean(&self) -> $t {
				*self.mean.borrow_mut().get_or_insert_with(|| {
					let series = self.series.borrow();
					series.iter().sum::<$t>() / (series.len() as $t)
				})
			}

			#[must_use]
			pub fn max(&self) -> $t {
				*self.max.borrow_mut().get_or_insert_with(|| {
					let series = self.series.borrow();
					*series.iter().max_by(|a, b| {
						a.total_cmp(b)
					}).unwrap()
				})
			}

			#[must_use]
			pub fn mid_range(&self) -> $t {
				(self.max() + self.min()) / 2.
			}

			#[must_use]
			pub fn min(&self) -> $t {
				*self.min.borrow_mut().get_or_insert_with(|| {
					let series = self.series.borrow();
					*series.iter().min_by(|a, b| {
						a.total_cmp(b)
					}).unwrap()
				})
			}

			#[must_use]
			pub fn median(&self) -> $t {
				let series = self.series.borrow();
				let len = series.len();
				if len.is_even() {
					(series[len / 2 - 1] + series[len / 2]) / (2. as $t)
				} else {
					series[len / 2]
				}
			}

			#[must_use]
			pub fn variance(&self) -> $t {
				*self.variance.borrow_mut().get_or_insert_with(|| {
					let series = self.series.borrow();
					self.series
						.borrow()
						.iter()
						.map(|v| (self.mean() - v).powi(2))
						.sum::<$t>()
						/ (series.len() as $t)
				})
			}

			#[must_use]
			pub fn std_dev(&self) -> $t {
				self.variance().sqrt()
			}
		}

	};
	($t:ty, $($others:ty),+) => {
		impl_statistics_for!($t);
		impl_statistics_for!($($others),+);
	};
}

impl_statistics_for!(f32, f64);

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_standard_deviation() {
		let values: &[f32] = &[2., 4., 4., 4., 5., 5., 7., 9.];
		let stats = SeriesStatistics::new_f32(values).unwrap();
		assert!((stats.std_dev() - 2.).abs() < f32::EPSILON);
	}

	#[test]
	fn test_mean() {
		let values: &[f64] = &[1., 3., 3., 6., 7., 8., 9.];
		let stats = SeriesStatistics::new_f64(values).unwrap();
		assert!((stats.mean() - 5.28).abs() < 0.01);
	}

	#[test]
	fn test_median() {
		let values: &[f64] = &[1., 3., 3., 6., 7., 8., 9.];
		let stats = SeriesStatistics::new_f64(values).unwrap();
		assert!((stats.median() - 6.).abs() < f64::EPSILON);
	}

	#[test]
	fn test_median_even_series() {
		let values: &[f64] = &[1., 2., 3., 4., 5., 6., 8., 9.];
		let stats = SeriesStatistics::new_f64(values).unwrap();
		assert!((stats.median() - 4.5).abs() < f64::EPSILON);
	}

	#[test]
	fn test_min() {
		let values: &[f64] = &[1., 2., 3., 4., 5., 6., 8., 9.];
		let stats = SeriesStatistics::new_f64(values).unwrap();
		assert!((stats.min() - 1.).abs() < f64::EPSILON);
	}

	#[test]
	fn test_max() {
		let values: &[f64] = &[1., 2., 3., 4., 5., 6., 8., 9.];
		let stats = SeriesStatistics::new_f64(values).unwrap();
		assert!((stats.max() - 9.).abs() < f64::EPSILON);
	}
}

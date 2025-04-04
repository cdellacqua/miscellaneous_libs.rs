use math_utils::discrete_interval::DiscreteInterval;

#[derive(Debug, Clone, Copy)]
pub struct DftCtx {
	sample_rate: usize,
	samples_per_window: usize,
}

impl DftCtx {
	#[must_use]
	pub const fn new(sample_rate: usize, samples_per_window: usize) -> Self {
		Self {
			sample_rate,
			samples_per_window,
		}
	}

	#[must_use]
	pub const fn sample_rate(&self) -> usize {
		self.sample_rate
	}

	#[must_use]
	pub const fn samples_per_window(&self) -> usize {
		self.samples_per_window
	}

	/// A [`DiscreteInterval`] instance that describes the DFT bins
	/// as a sequence of bins, centered around their respective frequencies.
	///
	/// Note that bin 0 is centered at 0Hz, which implies that it's range is from `(-bin_width / 2, +bin_width / 2)`.
	/// Also note that this discrete interval includes the Nyquist frequency (`bin == samples_per_window / 2`), which is centered around `sample_rate / 2`, therefore
	/// its range is `(sample_rate / 2 - bin_width / 2, sample_rate / 2 + bin_width / 2)`.
	#[must_use]
	#[allow(clippy::cast_precision_loss)]
	pub fn frequency_interval(&self) -> DiscreteInterval<f32> {
		DiscreteInterval::new(
			(
				-(self.sample_rate as f32 / 2. / self.samples_per_window as f32),
				self.sample_rate as f32 / 2.
					+ (self.sample_rate as f32 / 2. / self.samples_per_window as f32),
			),
			self.n_of_bins(),
		)
	}

	#[must_use]
	pub fn frequency_to_bin(&self, frequency: f32) -> usize {
		self.frequency_interval().value_to_bin(frequency)
	}

	#[must_use]
	pub fn frequency_gap(&self) -> f32 {
		self.frequency_interval().bin_width()
	}

	#[must_use]
	pub fn bin_to_frequency(&self, bin: usize) -> f32 {
		self.frequency_interval().bin_midpoint(bin)
	}

	#[must_use]
	pub fn bin_frequency_interval(&self, bin: usize) -> (f32, f32) {
		self.frequency_interval().bin_range(bin)
	}

	#[must_use]
	pub fn bins(&self) -> Vec<usize> {
		(0..self.n_of_bins()).collect()
	}

	/// DFT results are mirrored.
	///
	/// When `samples_per_window == sample_rate`, the range includes all the indices that correspond to
	/// the frequencies between 0 and the Nyquist frequency.
	#[must_use]
	pub const fn n_of_bins(&self) -> usize {
		self.samples_per_window / 2 + 1
	}
}

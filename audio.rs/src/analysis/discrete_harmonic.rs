use std::fmt::Debug;

use rustfft::num_complex::Complex32;

use super::DiscreteFrequency;

#[derive(Clone, Copy, PartialEq, Default)]
pub struct DiscreteHarmonic {
	sample_rate: usize,
	samples_per_window: usize,
	pub(crate) phasor: Complex32,
	frequency_bin: DiscreteFrequency,
}

impl Debug for DiscreteHarmonic {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("DiscreteHarmonic")
			.field("sample_rate", &self.sample_rate)
			.field("samples_per_window", &self.samples_per_window)
			.field("phasor", &self.phasor)
			.field("frequency_bin", &self.frequency_bin)
			.field("power()", &self.power())
			.field("phase()", &self.phase())
			.finish()
	}
}

impl DiscreteHarmonic {
	#[must_use]
	pub fn new(
		sample_rate: usize,
		samples_per_window: usize,
		phasor: Complex32,
		frequency_bin: DiscreteFrequency,
	) -> Self {
		Self {
			sample_rate,
			samples_per_window,
			phasor,
			frequency_bin,
		}
	}

	#[must_use]
	pub fn from_bin_idx(
		sample_rate: usize,
		samples_per_window: usize,
		phasor: Complex32,
		bin_idx: usize,
	) -> Self {
		Self {
			sample_rate,
			samples_per_window,
			phasor,
			frequency_bin: DiscreteFrequency::new(sample_rate, samples_per_window, bin_idx),
		}
	}

	#[must_use]
	pub fn from_frequency(
		sample_rate: usize,
		samples_per_window: usize,
		phasor: Complex32,
		frequency: f32,
	) -> Self {
		Self {
			sample_rate,
			samples_per_window,
			phasor,
			frequency_bin: DiscreteFrequency::from_frequency(sample_rate, samples_per_window, frequency),
		}
	}

	#[must_use]
	pub fn frequency(&self) -> f32 {
		self.frequency_bin.frequency()
	}

	#[must_use]
	pub const fn bin_idx(&self) -> usize {
		self.frequency_bin.bin_idx()
	}

	#[must_use]
	pub const fn frequency_bin(&self) -> DiscreteFrequency {
		self.frequency_bin
	}

	#[must_use]
	pub const fn sample_rate(&self) -> usize {
		self.sample_rate
	}

	#[must_use]
	pub const fn samples_per_window(&self) -> usize {
		self.samples_per_window
	}

	/// The phase of the harmonic represents the phase offset of a cosine wave (i.e. the real component of a DFT point).
	#[must_use]
	pub fn phase(&self) -> f32 {
		self.phasor.arg()
	}

	#[must_use]
	pub fn amplitude(&self) -> f32 {
		self.phasor.norm()
	}

	/// The value returned by this method is unitless and represents
	/// the energy of the harmonic over the sampling period.
	#[must_use]
	pub fn power(&self) -> f32 {
		// Electrically speaking:
		//
		// P = V²/R
		//
		// where P is power, V is voltage and R is resistance.
		self.phasor.norm_sqr()
	}

	/// Get the underlying complex number representing the
	/// phase and amplitude of this harmonic.
	#[must_use]
	pub fn phasor(&self) -> Complex32 {
		self.phasor
	}
}

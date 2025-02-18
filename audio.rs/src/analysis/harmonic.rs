use rustfft::num_complex::Complex32;

use super::FrequencyBin;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Harmonic<const SAMPLE_RATE: usize, const SAMPLES: usize> {
	phasor: Complex32,
	frequency_bin: FrequencyBin<SAMPLE_RATE, SAMPLES>,
}

impl<const SAMPLE_RATE: usize, const SAMPLES: usize> Harmonic<SAMPLE_RATE, SAMPLES> {
	#[must_use]
	pub fn new(phasor: Complex32, frequency_bin: FrequencyBin<SAMPLE_RATE, SAMPLES>) -> Self {
		Self {
			phasor,
			frequency_bin,
		}
	}

	#[must_use]
	pub fn from_bin_idx(phasor: Complex32, bin_idx: usize) -> Self {
		Self {
			phasor,
			frequency_bin: FrequencyBin::new(bin_idx),
		}
	}

	#[must_use]
	pub fn from_frequency(phasor: Complex32, frequency: f32) -> Self {
		Self {
			phasor,
			frequency_bin: FrequencyBin::from_frequency(frequency),
		}
	}

	#[must_use]
	pub const fn frequency(&self) -> f32 {
		self.frequency_bin.frequency()
	}

	#[must_use]
	pub const fn bin_idx(&self) -> usize {
		self.frequency_bin.bin_idx()
	}

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

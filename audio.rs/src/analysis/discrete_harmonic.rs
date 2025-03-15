use std::fmt::Debug;

use rustfft::num_complex::Complex32;

use super::FrequencyBin;

#[derive(Clone, Copy, PartialEq, Default)]
pub struct DiscreteHarmonic<const SAMPLE_RATE: usize, const SAMPLES_PER_WINDOW: usize> {
	phasor: Complex32,
	frequency_bin: FrequencyBin<SAMPLE_RATE, SAMPLES_PER_WINDOW>,
}

impl<const SAMPLE_RATE: usize, const SAMPLES_PER_WINDOW: usize> Debug
	for DiscreteHarmonic<SAMPLE_RATE, SAMPLES_PER_WINDOW>
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct(&format!(
			"DiscreteHarmonic<{SAMPLE_RATE}, {SAMPLES_PER_WINDOW}>"
		))
		.field("phasor", &self.phasor)
		.field("frequency_bin", &self.frequency_bin)
		.field("power()", &self.power())
		.field("phase()", &self.phase())
		.finish()
	}
}

impl<const SAMPLE_RATE: usize, const SAMPLES_PER_WINDOW: usize>
	DiscreteHarmonic<SAMPLE_RATE, SAMPLES_PER_WINDOW>
{
	#[must_use]
	pub fn new(
		phasor: Complex32,
		frequency_bin: FrequencyBin<SAMPLE_RATE, SAMPLES_PER_WINDOW>,
	) -> Self {
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
	pub fn frequency(&self) -> f32 {
		self.frequency_bin.frequency()
	}

	#[must_use]
	pub const fn bin_idx(&self) -> usize {
		self.frequency_bin.bin_idx()
	}

	#[must_use]
	pub const fn frequency_bin(&self) -> FrequencyBin<SAMPLE_RATE, SAMPLES_PER_WINDOW> {
		self.frequency_bin
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

use std::fmt::Debug;

use rustfft::num_complex::Complex32;

#[derive(Clone, Copy, PartialEq, Default)]
pub struct Harmonic {
	phasor: Complex32,
	frequency: f32,
}

impl Debug for Harmonic {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Harmonic")
			.field("phasor", &self.phasor)
			.field("frequency", &self.frequency)
			.field("power()", &self.power())
			.field("phase()", &self.phase())
			.finish()
	}
}

impl Harmonic {
	#[must_use]
	pub fn new(phasor: Complex32, frequency: f32) -> Self {
		Self { phasor, frequency }
	}

	#[must_use]
	pub const fn frequency(&self) -> f32 {
		self.frequency
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

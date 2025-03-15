pub mod dft;

mod windowing_fn;
pub use windowing_fn::*;

pub mod windowing_fns;

mod discrete_harmonic;
pub use discrete_harmonic::*;

mod harmonic;
pub use harmonic::*;

mod frequency_bin;
pub use frequency_bin::*;

impl<const SAMPLE_RATE: usize, const SAMPLES_PER_WINDOW: usize> From<Harmonic>
	for DiscreteHarmonic<SAMPLE_RATE, SAMPLES_PER_WINDOW>
{
	fn from(value: Harmonic) -> Self {
		Self::from_frequency(value.phasor(), value.frequency())
	}
}

impl<const SAMPLE_RATE: usize, const SAMPLES_PER_WINDOW: usize>
	From<DiscreteHarmonic<SAMPLE_RATE, SAMPLES_PER_WINDOW>> for Harmonic
{
	fn from(value: DiscreteHarmonic<SAMPLE_RATE, SAMPLES_PER_WINDOW>) -> Self {
		Self::new(value.phasor(), value.frequency())
	}
}

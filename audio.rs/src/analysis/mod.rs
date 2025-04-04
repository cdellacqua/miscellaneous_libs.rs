pub mod dft;

mod windowing_fn;
pub use windowing_fn::*;

pub mod windowing_fns;

mod discrete_harmonic;
pub use discrete_harmonic::*;

mod harmonic;
pub use harmonic::*;

mod discrete_frequency;
pub use discrete_frequency::*;

impl From<DiscreteHarmonic> for Harmonic {
	fn from(value: DiscreteHarmonic) -> Self {
		Self::new(
			value.phasor(),
			value.frequency(),
		)
	}
}

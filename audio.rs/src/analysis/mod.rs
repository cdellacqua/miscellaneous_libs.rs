pub mod dft;

mod windowing_fn;
pub use windowing_fn::*;

pub mod windowing_fns;

mod harmonic;
pub use harmonic::*;

mod discrete_harmonic;
pub use discrete_harmonic::*;

mod dft_ctx;
pub use dft_ctx::*;

impl DiscreteHarmonic {
	#[must_use]
	pub fn to_harmonic(&self, dft_ctx: DftCtx) -> Harmonic {
		Harmonic::new(self.phasor(), dft_ctx.bin_to_frequency(self.bin()))
	}
}

impl Harmonic {
	#[must_use]
	pub fn to_discrete_harmonic(&self, dft_ctx: DftCtx) -> DiscreteHarmonic {
		DiscreteHarmonic::new(self.phasor(), dft_ctx.frequency_to_bin(self.frequency()))
	}
}

use std::fmt::Display;

use derive_more::derive::{
	Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign,
};

#[derive(
	Debug,
	Clone,
	Copy,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	Default,
	Hash,
	Add,
	AddAssign,
	Sub,
	SubAssign,
	Div,
	DivAssign,
	Mul,
	MulAssign,
	Rem,
	RemAssign,
)]
pub struct SampleRate(pub usize);

impl Display for SampleRate {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Display::fmt(&format!("{}Hz", self.0), f)
	}
}

impl From<usize> for SampleRate {
	fn from(value: usize) -> Self {
		Self(value)
	}
}

impl From<SampleRate> for usize {
	fn from(value: SampleRate) -> Self {
		value.0
	}
}

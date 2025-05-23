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
pub struct NOfFrames(pub usize);

impl Display for NOfFrames {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Display::fmt(&self.0, f)
	}
}

impl From<usize> for NOfFrames {
	fn from(value: usize) -> Self {
		Self(value)
	}
}

impl From<NOfFrames> for usize {
	fn from(value: NOfFrames) -> Self {
		value.0
	}
}

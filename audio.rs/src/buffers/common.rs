pub trait ToMono {
	type Target;

	fn to_mono(&self) -> Self::Target;
}

pub trait NOfChannels {
	fn n_of_channels(&self) -> usize;
}

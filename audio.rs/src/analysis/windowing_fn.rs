pub trait WindowingFn {
	fn ratio_at(&mut self, index: usize, n_of_samples: usize) -> f32;
}

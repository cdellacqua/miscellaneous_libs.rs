pub trait WindowingFn {
	fn ratio_at(&self, sample_idx: usize, n_of_samples: usize) -> f32;
}

use std::time::Duration;

use buffer_hopper::BufferHopper;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn test_impl(c: &mut Criterion) {
	use rand::prelude::*;
	let mut rng = rand::thread_rng();
	let buffer: Vec<i32> = (0..10_000)
		.map(|_| rng.gen_range(-10_000..=-10_000))
		.collect();

	let mut hopper = BufferHopper::new_with_overlap(100, 50);
	c.bench_function("test_impl", |b| {
		b.iter(|| {
			hopper.feed(&*buffer, |_, _| {
				black_box(());
			});
			black_box(());
		});
	});
}

criterion_group! {
  name = benches;
  config = Criterion::default().measurement_time(Duration::from_secs(8));
  targets = test_impl
}
criterion_main!(benches);

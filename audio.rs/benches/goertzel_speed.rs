use std::time::Duration;

use audio::analysis::{dft::GoertzelAnalyzer, windowing_fns::HannWindow};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_goertzel(c: &mut Criterion) {
	use rand::prelude::*;
	let mut rng = rand::thread_rng();
	let sample: Vec<f32> = (0..64).map(|_| rng.gen_range(-1.0..=1.0)).collect();

	let mut analyzer = GoertzelAnalyzer::<44_100, 64>::new(
		vec![2.into(), 3.into(), 4.into(), 5.into()],
		&HannWindow::new(),
	);
	c.bench_function("Goertzel analyzer", |b| {
		b.iter(|| {
			black_box(analyzer.analyze(&sample));
		});
	});
}

criterion_group! {
  name = benches;
  config = Criterion::default().measurement_time(Duration::from_secs(8));
  targets = bench_goertzel
}
criterion_main!(benches);

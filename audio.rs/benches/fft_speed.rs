use std::time::Duration;

use audio::analysis::{dft::StftAnalyzer, windowing_fns::HannWindow};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_fft_impls(c: &mut Criterion) {
	use rand::prelude::*;
	let mut rng = rand::thread_rng();
	let sample: Vec<f32> = (0..44_100 * 10)
		.map(|_| rng.gen_range(-1.0..=1.0))
		.collect();

	let mut group = c.benchmark_group("FFT Implementations");

	// let mut analyzer = fft::FftAnalyzer::new(44_100, (0., 20_000.), hann_window);
	// group.bench_function(BenchmarkId::new("Naive", "sample"), |b| {
	// 	b.iter(|| {
	// 		black_box(analyzer.analyze(&sample));
	// 	});
	// });

	let mut analyzer = StftAnalyzer::<44_100, 10>::new(HannWindow);
	group.bench_function(BenchmarkId::new("Optimized allocations", "sample"), |b| {
		b.iter(|| {
			black_box(analyzer.analyze_bins(&sample));
		});
	});

	group.finish();
}

criterion_group! {
  name = benches;
  config = Criterion::default().measurement_time(Duration::from_secs(8));
  targets = bench_fft_impls
}
criterion_main!(benches);

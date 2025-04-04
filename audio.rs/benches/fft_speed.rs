use std::time::Duration;

use audio::{analysis::{dft::StftAnalyzer, windowing_fns::HannWindow, DftCtx}, SampleRate};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_fft_impls(c: &mut Criterion) {
	use rand::prelude::*;
	let mut rng = rand::thread_rng();
	let samples: Vec<f32> = (0..64).map(|_| rng.gen_range(-1.0..=1.0)).collect();

	let mut group = c.benchmark_group("FFT Implementations");

	// let mut analyzer = fft::FftAnalyzer::new(44_100, (0., 20_000.), hann_window);
	// group.bench_function(BenchmarkId::new("Naive", "sample"), |b| {
	// 	b.iter(|| {
	// 		black_box(analyzer.analyze(&sample));
	// 	});
	// });

	let mut analyzer = StftAnalyzer::new(DftCtx::new(SampleRate(44_100), 64), &HannWindow);
	group.bench_function(BenchmarkId::new("Optimized allocations", "sample"), |b| {
		b.iter(|| {
			black_box(analyzer.analyze(&samples));
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

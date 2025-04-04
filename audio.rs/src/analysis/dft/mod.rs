mod stft_analyzer;
pub use stft_analyzer::*;

mod goertzel_analyzer;
pub use goertzel_analyzer::*;

#[cfg(test)]
#[cfg(feature = "output")]
mod tests {
	use std::f32::consts::TAU;

	use rustfft::num_complex::Complex32;

	use crate::{
		analysis::{
			dft::{GoertzelAnalyzer, StftAnalyzer},
			windowing_fns::HannWindow,
			DftCtx, Harmonic,
		},
		output::harmonics_to_samples, SampleRate,
	};

	#[test]
	fn cross_check_goertzel_and_stft() {
		let dft_ctx = DftCtx::new(SampleRate(44100), 44100);

		let frequency = 440.;
		let frequency_bin = dft_ctx.frequency_to_bin(frequency);

		let signal = harmonics_to_samples(
			dft_ctx.sample_rate(),
			dft_ctx.samples_per_window(),
			&[Harmonic::new(Complex32::ONE, frequency)],
		);
		let mut goertzel = GoertzelAnalyzer::new(
			dft_ctx,
			vec![
				frequency_bin - 20,
				frequency_bin - 15,
				frequency_bin - 10,
				frequency_bin - 5,
				frequency_bin,
				frequency_bin + 5,
				frequency_bin + 10,
				frequency_bin + 15,
				frequency_bin + 20,
			],
			&HannWindow::new(),
		);
		let mut stft = StftAnalyzer::new(dft_ctx, &HannWindow::new());

		let stft_result = stft
			.analyze(&signal)
			.iter()
			.max_by(|a, b| a.power().total_cmp(&b.power()))
			.unwrap();
		let goertzel_result = goertzel
			.analyze(&signal)
			.iter()
			.max_by(|a, b| a.power().total_cmp(&b.power()))
			.unwrap();

		assert_eq!(
			stft_result.bin(),
			goertzel_result.bin(),
			"goertzel and stft should yield the same frequency result"
		);
		assert!(
			(stft_result.amplitude() - goertzel_result.amplitude()).abs() < 0.01,
			"goertzel and stft should yield a similar amplitude result"
		);
		assert!(
			(stft_result.phase() - goertzel_result.phase()).abs() < TAU / 100.,
			"goertzel and stft should yield a similar phase result"
		);
	}
}

mod stft_analyzer;
pub use stft_analyzer::*;

mod goertzel_analyzer;
pub use goertzel_analyzer::*;

#[cfg(test)]
mod tests {
	use std::f32::consts::TAU;

	use crate::{
		analysis::{
			dft::{GoertzelAnalyzer, StftAnalyzer},
			windowing_fns::HannWindow,
			FrequencyBin,
		},
		output::frequencies_to_samples,
	};

	#[test]
	fn cross_check_goertzel_and_stft() {
		const SAMPLE_RATE: usize = 44100;
		const SAMPLES: usize = 44100;

		let frequency = 440.;
		let frequency_bin = FrequencyBin::<SAMPLE_RATE, SAMPLES>::from_frequency(frequency);

		let signal = frequencies_to_samples::<SAMPLE_RATE>(SAMPLES, &[frequency]);
		let signal = signal.as_mono();
		let mut goertzel: GoertzelAnalyzer<SAMPLE_RATE, SAMPLES> = GoertzelAnalyzer::new(
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
			HannWindow::new(),
		);
		let mut stft: StftAnalyzer<SAMPLE_RATE, SAMPLES> = StftAnalyzer::new(HannWindow::new());

		let stft_result = stft
			.analyze_bins(signal)
			.iter()
			.max_by(|a, b| a.norm_sqr().total_cmp(&b.norm_sqr()))
			.unwrap();
		let goertzel_result = goertzel
			.analyze_bins(signal)
			.iter()
			.max_by(|a, b| a.norm_sqr().total_cmp(&b.norm_sqr()))
			.unwrap();

		assert_eq!(
			stft_result.frequency_bin, goertzel_result.frequency_bin,
			"goertzel and stft should yield the same frequency result"
		);
		assert!(
			(stft_result.c.norm() - goertzel_result.c.norm()).abs() < 0.01,
			"goertzel and stft should yield a similar magnitude result"
		);
		assert!(
			(stft_result.c.arg() - goertzel_result.c.arg()).abs() < TAU / 100.,
			"goertzel and stft should yield a similar phase result"
		);
	}
}

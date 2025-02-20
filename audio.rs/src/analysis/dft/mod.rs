mod stft_analyzer;
pub use stft_analyzer::*;

mod goertzel_analyzer;
pub use goertzel_analyzer::*;

#[cfg(test)]
#[cfg(feature = "output")]
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
		let mut goertzel: GoertzelAnalyzer<SAMPLE_RATE, SAMPLES, _> = GoertzelAnalyzer::new(
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
		let mut stft: StftAnalyzer<SAMPLE_RATE, SAMPLES, _> = StftAnalyzer::new(HannWindow::new());

		let stft_result = stft
			.analyze(signal)
			.iter()
			.max_by(|a, b| a.power().total_cmp(&b.power()))
			.unwrap();
		let goertzel_result = goertzel
			.analyze(signal)
			.iter()
			.max_by(|a, b| a.power().total_cmp(&b.power()))
			.unwrap();

		assert_eq!(
			stft_result.bin_idx(),
			goertzel_result.bin_idx(),
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

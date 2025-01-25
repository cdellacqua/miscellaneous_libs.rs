#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use std::{f32::consts::TAU, iter};

use crate::{
	buffers::{AudioFrame, InterleavedAudioBuffer},
	AudioStreamBuilderError, AudioStreamSamplingState, NOfSamples,
};

use super::playback::{AudioPlayer, AudioPlayerBuilder};

/* TODO: support different set of frequencies per channel? */
#[derive(Debug, Clone)]
pub struct OscillatorBuilder<const SAMPLE_RATE: usize, const N_CH: usize> {
	frequencies: Vec<f32>,
	mute: bool,
	player_builder: AudioPlayerBuilder<SAMPLE_RATE, N_CH>,
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> Default for OscillatorBuilder<SAMPLE_RATE, N_CH> {
	fn default() -> Self {
		Self::new(&[], false)
	}
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> OscillatorBuilder<SAMPLE_RATE, N_CH> {
	#[must_use]
	pub fn new(frequencies: &[f32], mute: bool) -> Self {
		Self {
			frequencies: frequencies.to_vec(),
			mute,
			player_builder: AudioPlayerBuilder::new(),
		}
	}

	/// Build and start output stream
	///
	/// # Errors
	/// [`AudioStreamBuilderError`]
	///
	/// # Panics
	/// - if the output device default configuration doesn't use f32 as the sample format.
	pub fn build(&self) -> Result<Oscillator<SAMPLE_RATE, N_CH>, AudioStreamBuilderError> {
		let player = self.player_builder.build()?;

		Ok(Oscillator::new(player, self.frequencies.clone(), self.mute))
	}
}

pub struct Oscillator<const SAMPLE_RATE: usize, const N_CH: usize> {
	frequencies: Vec<f32>,
	mute: bool,
	player: AudioPlayer<SAMPLE_RATE, N_CH>,
}

impl<const SAMPLE_RATE: usize, const N_CH: usize> Oscillator<SAMPLE_RATE, N_CH> {
	fn new(mut player: AudioPlayer<SAMPLE_RATE, N_CH>, frequencies: Vec<f32>, mute: bool) -> Self {
		let signal = Self::generate_signal(&frequencies, mute);
		player.set_signal(signal);
		Self {
			frequencies,
			mute,
			player,
		}
	}

	#[must_use]
	pub fn state(&self) -> AudioStreamSamplingState {
		self.player.state()
	}

	pub fn stop(&mut self) {
		self.player.stop();
	}

	fn generate_signal(
		frequencies: &[f32],
		mute: bool,
	) -> Box<dyn Iterator<Item = AudioFrame<N_CH, [f32; N_CH]>> + Send + Sync> {
		if mute || frequencies.is_empty() {
			Box::new(iter::empty())
		} else {
			let frequencies = frequencies.to_vec();

			let mono =
				frequencies_to_samples::<SAMPLE_RATE>(NOfSamples::new(SAMPLE_RATE), &frequencies);

			Box::new(
				mono.into_iter()
					.cycle()
					.map(|frame| AudioFrame::new([frame[0]; N_CH])),
			)
		}
	}

	pub fn set_frequencies(&mut self, frequencies: &[f32]) {
		self.frequencies = frequencies.to_vec();
		self.player
			.set_signal(Self::generate_signal(&self.frequencies, self.mute));
	}

	#[must_use]
	pub fn frequencies(&mut self) -> Vec<f32> {
		self.frequencies.clone()
	}

	pub fn set_mute(&mut self, mute: bool) {
		self.mute = mute;
		self.player
			.set_signal(Self::generate_signal(&self.frequencies, self.mute));
	}

	#[must_use]
	pub fn mute(&mut self) -> bool {
		self.mute
	}

	#[must_use]
	pub fn sample_rate(&self) -> usize {
		SAMPLE_RATE
	}

	#[must_use]
	pub fn n_of_channels(&self) -> usize {
		N_CH
	}
}

#[must_use]
pub fn frequencies_to_samples<const SAMPLE_RATE: usize>(
	samples: NOfSamples<SAMPLE_RATE>,
	frequencies: &[f32],
) -> InterleavedAudioBuffer<SAMPLE_RATE, 1, Vec<f32>> {
	let mut mono = (0..*samples)
		.map(move |i| {
			#[allow(clippy::cast_precision_loss)]
			frequencies
				.iter()
				.map(|f| f32::sin(TAU * f * (i as f32 / SAMPLE_RATE as f32)))
				.sum::<f32>()
		})
		.collect::<Vec<f32>>();

	let &abs_max = mono
		.iter()
		.max_by(|a, b| a.abs().total_cmp(&b.abs()))
		.unwrap_or(&1.);

	mono.iter_mut().for_each(|s| *s /= abs_max);

	InterleavedAudioBuffer::new(mono)
}

#[cfg(test)]
mod tests {
	use std::{thread::sleep, time::Duration};

	use super::OscillatorBuilder;

	#[test]
	fn test_440() {
		let _oscillator = OscillatorBuilder::<44100, 1>::new(&[440.], false)
			.build()
			.unwrap();
		sleep(Duration::from_secs(1));
	}
	#[test]
	fn test_440_333() {
		let _oscillator = OscillatorBuilder::<44100, 1>::new(&[440., 333.], false)
			.build()
			.unwrap();
		sleep(Duration::from_secs(1));
	}
}

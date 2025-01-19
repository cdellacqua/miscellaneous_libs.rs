#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use std::f32::consts::TAU;

use crate::{buffers::AudioFrame, AudioStreamBuilderError, AudioStreamSamplingState};

use super::playback::{AudioPlayer, AudioPlayerBuilder};

/* TODO: support different set of frequencies per channel? */
#[derive(Debug, Clone)]
pub struct OscillatorBuilder<const N_CH: usize> {
	frequencies: Vec<f32>,
	mute: bool,
	player_builder: AudioPlayerBuilder<N_CH>,
}

impl<const N_CH: usize> Default for OscillatorBuilder<N_CH> {
	fn default() -> Self {
		Self::new(44100, &[], false)
	}
}

impl<const N_CH: usize> OscillatorBuilder<N_CH> {
	#[must_use]
	pub fn new(sample_rate: usize, frequencies: &[f32], mute: bool) -> Self {
		Self {
			frequencies: frequencies.to_vec(),
			mute,
			player_builder: AudioPlayerBuilder::new(sample_rate),
		}
	}

	/// Build and start output stream
	///
	/// # Errors
	/// [`AudioStreamBuilderError`]
	///
	/// # Panics
	/// - if the output device default configuration doesn't use f32 as the sample format.
	pub fn build(&self) -> Result<Oscillator<N_CH>, AudioStreamBuilderError> {
		let player = self.player_builder.build()?;

		Ok(Oscillator::new(player, self.frequencies.clone(), self.mute))
	}
}

pub struct Oscillator<const N_CH: usize> {
	sample_rate: usize,
	frequencies: Vec<f32>,
	mute: bool,
	player: AudioPlayer<N_CH>,
}

impl<const N_CH: usize> Oscillator<N_CH> {
	fn new(mut player: AudioPlayer<N_CH>, frequencies: Vec<f32>, mute: bool) -> Self {
		let sample_rate = player.sample_rate();

		let signal = Self::generate_signal(sample_rate, &frequencies, mute);
		player.set_signal(signal);
		Self {
			sample_rate,
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
		sample_rate: usize,
		frequencies: &[f32],
		mute: bool,
	) -> Box<dyn Iterator<Item = AudioFrame<N_CH, [f32; N_CH]>> + Send + Sync> {
		if mute || frequencies.is_empty() {
			Box::new(
				(0..sample_rate)
					.cycle()
					.map(move |_| AudioFrame::new([0.; N_CH])),
			)
		} else {
			let frequencies = frequencies.to_vec();

			let mut mono = (0..sample_rate)
				.map(move |i| {
					frequencies
						.iter()
						.map(|f| f32::sin(TAU * f * (i as f32 / sample_rate as f32)))
						.sum::<f32>()
				})
				.collect::<Vec<f32>>();

			let &abs_max = mono
				.iter()
				.max_by(|a, b| a.abs().total_cmp(&b.abs()))
				.unwrap_or(&1.);

			mono.iter_mut().for_each(|s| *s /= abs_max);

			Box::new(
				(0..sample_rate)
					.cycle()
					.map(move |i| AudioFrame::new([mono[i]; N_CH])),
			)
		}
	}

	pub fn set_frequencies(&mut self, frequencies: &[f32]) {
		self.frequencies = frequencies.to_vec();
		self.player.set_signal(Self::generate_signal(
			self.sample_rate,
			&self.frequencies,
			self.mute,
		));
	}

	#[must_use]
	pub fn frequencies(&mut self) -> Vec<f32> {
		self.frequencies.clone()
	}

	pub fn set_mute(&mut self, mute: bool) {
		self.mute = mute;
		self.player.set_signal(Self::generate_signal(
			self.sample_rate,
			&self.frequencies,
			self.mute,
		));
	}

	#[must_use]
	pub fn mute(&mut self) -> bool {
		self.mute
	}

	#[must_use]
	pub fn sample_rate(&self) -> usize {
		self.sample_rate
	}

	#[must_use]
	pub fn n_of_channels(&self) -> usize {
		N_CH
	}
}

#[cfg(test)]
mod tests {
	use std::{thread::sleep, time::Duration};

	use super::OscillatorBuilder;

	#[test]
	fn test_440() {
		let _oscillator = OscillatorBuilder::<1>::new(44100, &[440.], false)
			.build()
			.unwrap();
		sleep(Duration::from_secs(1));
	}
	#[test]
	fn test_440_333() {
		let _oscillator = OscillatorBuilder::<1>::new(44100, &[440., 333.], false)
			.build()
			.unwrap();
		sleep(Duration::from_secs(1));
	}
}

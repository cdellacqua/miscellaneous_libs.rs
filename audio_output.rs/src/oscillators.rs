#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use crate::{AudioOutputBuilderError, AudioPlayer, AudioPlayerBuilder, SamplingState};
use std::f32::consts::TAU;

#[derive(Debug, Clone)]
pub struct OscillatorBuilder {
	frequencies: Vec<f32>,
	mute: bool,
	player_builder: AudioPlayerBuilder,
}

impl Default for OscillatorBuilder {
	fn default() -> Self {
		Self::new(&[], false)
	}
}

impl OscillatorBuilder {
	#[must_use]
	pub fn new(frequencies: &[f32], mute: bool) -> Self {
		Self {
			frequencies: frequencies.to_vec(),
			mute,
			player_builder: AudioPlayerBuilder::new(),
		}
	}

	///
	/// Build and start output stream
	///
	/// # Errors
	/// [`AudioOutputBuilderError`]
	///
	/// # Panics
	/// - if the output device default configuration doesn't use f32 as the sample format
	pub fn build(&self) -> Result<Oscillator, AudioOutputBuilderError> {
		let player = self.player_builder.build()?;

		Ok(Oscillator::new(player, self.frequencies.clone(), self.mute))
	}
}

pub struct Oscillator {
	pub sample_rate: usize,
	frequencies: Vec<f32>,
	mute: bool,
	pub n_of_channels: usize,
	player: AudioPlayer,
}

impl Oscillator {
	fn new(mut player: AudioPlayer, frequencies: Vec<f32>, mute: bool) -> Self {
		let n_of_channels = player.n_of_channels;
		let sample_rate = player.sample_rate;

		let track = Self::generate_track(&frequencies, sample_rate, mute);
		player.set_mono_track(track.into_iter());
		Self {
			sample_rate,
			frequencies,
			mute,
			n_of_channels,
			player,
		}
	}

	#[must_use]
	pub fn state(&self) -> SamplingState {
		self.player.state()
	}

	pub fn stop(&mut self) {
		self.player.stop();
	}

	fn generate_track(
		frequencies: &[f32],
		sample_rate: usize,
		mute: bool,
	) -> Box<dyn Iterator<Item = f32> + Send + 'static> {
		if mute || frequencies.is_empty() {
			Box::new((0..sample_rate).cycle().map(|_| 0.))
		} else {
			let frequencies = frequencies.to_vec();
			let wave_magnitude = 1. / frequencies.len().max(1) as f32;

			Box::new((0..sample_rate).cycle().map(move |i| {
				frequencies
					.iter()
					.map(|f| {
						f32::sin(TAU * f * (i as f32 / (sample_rate - 1) as f32)) * wave_magnitude
					})
					.sum::<f32>()
			}))
		}
	}

	pub fn set_frequencies(&mut self, frequencies: &[f32]) {
		self.frequencies = frequencies.to_vec();
		self.player.set_mono_track(Self::generate_track(
			&self.frequencies,
			self.sample_rate,
			self.mute,
		));
	}

	pub fn frequencies(&mut self) -> Vec<f32> {
		self.frequencies.clone()
	}

	pub fn set_mute(&mut self, mute: bool) {
		self.mute = mute;
		self.player.set_mono_track(Self::generate_track(
			&self.frequencies,
			self.sample_rate,
			self.mute,
		));
	}

	pub fn mute(&mut self) -> bool {
		self.mute
	}
}

#[cfg(test)]
mod tests {
	use std::{thread::sleep, time::Duration};

	use super::OscillatorBuilder;

	#[test]
	#[ignore]
	fn test_440() {
		let _oscillator = OscillatorBuilder::new(&[440.], false).build().unwrap();
		sleep(Duration::from_secs(10));
	}
	#[test]
	fn test_440_333() {
		let _oscillator = OscillatorBuilder::new(&[440., 333.], false)
			.build()
			.unwrap();
		sleep(Duration::from_secs(10));
	}
}

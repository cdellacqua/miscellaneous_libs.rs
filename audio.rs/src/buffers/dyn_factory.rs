use std::borrow::{Borrow, BorrowMut};

use super::{InterleavedAudioBuffer, InterleavedAudioBufferTrait, InterleavedAudioBufferTraitMut};

pub struct InterleavedAudioBufferFactory;

impl InterleavedAudioBufferFactory {
	pub fn build<Buffer: Borrow<[f32]> + 'static>(
		n_of_channels: usize,
		raw_buffer: Buffer,
	) -> Box<dyn InterleavedAudioBufferTrait> {
		match n_of_channels {
			1 => Box::new(InterleavedAudioBuffer::<1, Buffer>::new(raw_buffer)),
			2 => Box::new(InterleavedAudioBuffer::<2, Buffer>::new(raw_buffer)),
			3 => Box::new(InterleavedAudioBuffer::<3, Buffer>::new(raw_buffer)),
			4 => Box::new(InterleavedAudioBuffer::<4, Buffer>::new(raw_buffer)),
			5 => Box::new(InterleavedAudioBuffer::<5, Buffer>::new(raw_buffer)),
			6 => Box::new(InterleavedAudioBuffer::<6, Buffer>::new(raw_buffer)),
			7 => Box::new(InterleavedAudioBuffer::<7, Buffer>::new(raw_buffer)),
			8 => Box::new(InterleavedAudioBuffer::<8, Buffer>::new(raw_buffer)),
			9 => Box::new(InterleavedAudioBuffer::<9, Buffer>::new(raw_buffer)),
			10 => Box::new(InterleavedAudioBuffer::<10, Buffer>::new(raw_buffer)),
			11 => Box::new(InterleavedAudioBuffer::<11, Buffer>::new(raw_buffer)),
			12 => Box::new(InterleavedAudioBuffer::<12, Buffer>::new(raw_buffer)),
			13 => Box::new(InterleavedAudioBuffer::<13, Buffer>::new(raw_buffer)),
			14 => Box::new(InterleavedAudioBuffer::<14, Buffer>::new(raw_buffer)),
			15 => Box::new(InterleavedAudioBuffer::<15, Buffer>::new(raw_buffer)),
			16 => Box::new(InterleavedAudioBuffer::<16, Buffer>::new(raw_buffer)),
			17 => Box::new(InterleavedAudioBuffer::<17, Buffer>::new(raw_buffer)),
			18 => Box::new(InterleavedAudioBuffer::<18, Buffer>::new(raw_buffer)),
			19 => Box::new(InterleavedAudioBuffer::<19, Buffer>::new(raw_buffer)),
			20 => Box::new(InterleavedAudioBuffer::<20, Buffer>::new(raw_buffer)),
			21 => Box::new(InterleavedAudioBuffer::<21, Buffer>::new(raw_buffer)),
			22 => Box::new(InterleavedAudioBuffer::<22, Buffer>::new(raw_buffer)),
			_ => unimplemented!(),
		}
	}

	pub fn build_mut<Buffer: BorrowMut<[f32]> + 'static>(
		n_of_channels: usize,
		raw_buffer: Buffer,
	) -> Box<dyn InterleavedAudioBufferTraitMut> {
		match n_of_channels {
			1 => Box::new(InterleavedAudioBuffer::<1, Buffer>::new(raw_buffer)),
			2 => Box::new(InterleavedAudioBuffer::<2, Buffer>::new(raw_buffer)),
			3 => Box::new(InterleavedAudioBuffer::<3, Buffer>::new(raw_buffer)),
			4 => Box::new(InterleavedAudioBuffer::<4, Buffer>::new(raw_buffer)),
			5 => Box::new(InterleavedAudioBuffer::<5, Buffer>::new(raw_buffer)),
			6 => Box::new(InterleavedAudioBuffer::<6, Buffer>::new(raw_buffer)),
			7 => Box::new(InterleavedAudioBuffer::<7, Buffer>::new(raw_buffer)),
			8 => Box::new(InterleavedAudioBuffer::<8, Buffer>::new(raw_buffer)),
			9 => Box::new(InterleavedAudioBuffer::<9, Buffer>::new(raw_buffer)),
			10 => Box::new(InterleavedAudioBuffer::<10, Buffer>::new(raw_buffer)),
			11 => Box::new(InterleavedAudioBuffer::<11, Buffer>::new(raw_buffer)),
			12 => Box::new(InterleavedAudioBuffer::<12, Buffer>::new(raw_buffer)),
			13 => Box::new(InterleavedAudioBuffer::<13, Buffer>::new(raw_buffer)),
			14 => Box::new(InterleavedAudioBuffer::<14, Buffer>::new(raw_buffer)),
			15 => Box::new(InterleavedAudioBuffer::<15, Buffer>::new(raw_buffer)),
			16 => Box::new(InterleavedAudioBuffer::<16, Buffer>::new(raw_buffer)),
			17 => Box::new(InterleavedAudioBuffer::<17, Buffer>::new(raw_buffer)),
			18 => Box::new(InterleavedAudioBuffer::<18, Buffer>::new(raw_buffer)),
			19 => Box::new(InterleavedAudioBuffer::<19, Buffer>::new(raw_buffer)),
			20 => Box::new(InterleavedAudioBuffer::<20, Buffer>::new(raw_buffer)),
			21 => Box::new(InterleavedAudioBuffer::<21, Buffer>::new(raw_buffer)),
			22 => Box::new(InterleavedAudioBuffer::<22, Buffer>::new(raw_buffer)),
			_ => unimplemented!(),
		}
	}
}

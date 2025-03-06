//! `<num>::round` and similar operations is currently not available outside of const contexts.
//! This module provides some of those operations via helper functions.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

#[must_use]
#[allow(clippy::cast_sign_loss)]
#[inline]
pub const fn round_f32_to_usize(val: f32) -> usize {
	(val + 0.5) as usize
}

#[must_use]
#[allow(clippy::cast_sign_loss)]
#[inline]
pub const fn round_f64_to_usize(val: f64) -> usize {
	(val + 0.5) as usize
}

#[must_use]
#[inline]
pub const fn round_f32_to_isize(val: f32) -> isize {
	if val < 0. {
		-(round_f32_to_usize(-val) as isize)
	} else {
		round_f32_to_usize(val) as isize
	}
}

#[must_use]
pub const fn round_f64_to_isize(val: f64) -> isize {
	if val < 0. {
		-(round_f64_to_usize(-val) as isize)
	} else {
		round_f64_to_usize(val) as isize
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	#[allow(clippy::cast_sign_loss)]
	fn test_round_usize() {
		for i in 0..1000i16 {
			let t = f32::from(i) / 100.;
			assert_eq!(t.round() as usize, round_f32_to_usize(t));
		}
	}

	#[test]
	#[allow(clippy::cast_sign_loss)]
	fn test_round_isize() {
		for i in -1000..1000i16 {
			let t = f32::from(i) / 100.;
			assert_eq!(t.round() as isize, round_f32_to_isize(t));
		}
	}
}

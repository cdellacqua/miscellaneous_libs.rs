pub trait NextPowerOfTwo
where
	Self: Sized,
{
	/// Return the power of two that "follows" a given number.
	/// For positive numbers, the next power of 2 is always greater
	/// than the given number, e.g. `290.next_pow_of_2() == 512` and also
	/// `256.next_pow_of_2() == 512`.
	///
	/// For negative numbers, the same mechanism is applied to the corresponding
	/// positive value, then the negative sign is applied to the result, e.g.
	/// `(-256).next_pow_of_2() == -512`. In general,
	/// `(-x).next_pow_of_2() == -(x.next_pow_of_2())`
	///
	/// Example:
	///
	/// ```
	/// use math_utils::bit_manipulation::NextPowerOfTwo;
	///
	/// assert_eq!((1000_i32).next_pow_of_2(), 1024);
	/// assert_eq!((333_i32).next_pow_of_2(), 512);
	/// assert_eq!((257_i32).next_pow_of_2(), 512);
	/// assert_eq!((256_i32).next_pow_of_2(), 512);
	/// assert_eq!((255_i32).next_pow_of_2(), 256);
	/// assert_eq!((-255_i32).next_pow_of_2(), -256);
	/// assert_eq!((-256_i32).next_pow_of_2(), -512);
	/// assert_eq!((-257_i32).next_pow_of_2(), -512);
	/// assert_eq!((-333_i32).next_pow_of_2(), -512);
	/// assert_eq!((-1000_i32).next_pow_of_2(), -1024);
	/// ```
	#[must_use]
	fn next_pow_of_2(&self) -> Self;
}

macro_rules! impl_next_power_of_two_for_unsigned {
	($t:ty) => {
		impl NextPowerOfTwo for $t {
			fn next_pow_of_2(&self) -> Self {
				1 << (Self::BITS - self.leading_zeros())
			}
		}
	};
	($t:ty, $($others:ty),+) => {
		impl_next_power_of_two_for_unsigned!($t);
		impl_next_power_of_two_for_unsigned!($($others),+);
	};
}

macro_rules! impl_next_power_of_two_for_signed {
	($t:ty) => {
		impl NextPowerOfTwo for $t {
			fn next_pow_of_2(&self) -> Self {
				self.signum() * (1 << (Self::BITS - self.abs().leading_zeros()))
			}
		}
	};
	($t:ty, $($others:ty),+) => {
		impl_next_power_of_two_for_signed!($t);
		impl_next_power_of_two_for_signed!($($others),+);
	};
}

impl_next_power_of_two_for_unsigned!(u8, u16, u32, u64, u128, usize);
impl_next_power_of_two_for_signed!(i8, i16, i32, i64, i128, isize);

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_next_pow_of_2() {
		assert_eq!(1000.next_pow_of_2(), 1024);
		assert_eq!(333.next_pow_of_2(), 512);
		assert_eq!(257.next_pow_of_2(), 512);
		assert_eq!(256.next_pow_of_2(), 512);
		assert_eq!(255.next_pow_of_2(), 256);
	}

	#[test]
	fn test_next_pow_of_2_neg() {
		assert_eq!((-1000).next_pow_of_2(), -1024);
		assert_eq!((-333).next_pow_of_2(), -512);
		assert_eq!((-257).next_pow_of_2(), -512);
		assert_eq!((-256).next_pow_of_2(), -512);
		assert_eq!((-255).next_pow_of_2(), -256);
	}

	#[test]
	fn test_next_pow_of_2_mirror() {
		for i in (0..1000).step_by(7) {
			assert_eq!((-i).next_pow_of_2(), -(i.next_pow_of_2()));
		}
	}
}

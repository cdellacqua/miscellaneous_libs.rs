pub trait IsEven
where
	Self: Sized,
{
	#[must_use]
	fn is_even(&self) -> bool;
}

pub trait IsOdd
where
	Self: Sized,
{
	#[must_use]
	fn is_odd(&self) -> bool;
}
macro_rules! impl_is_even_is_odd_for {
	($t:ty) => {
		impl IsEven for $t {
			fn is_even(&self) -> bool {
				self & 1 == 0
			}
		}

		impl IsOdd for $t {
			fn is_odd(&self) -> bool {
				self & 1 == 1
			}
		}
	};
	($t:ty, $($others:ty),+) => {
		impl_is_even_is_odd_for!($t);
		impl_is_even_is_odd_for!($($others),+);
	};
}

impl_is_even_is_odd_for!(u8, u16, u32, u64, u128, isize, i8, i16, i32, i64, i128, usize);

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_a_bunch() {
		let mut toggle = true;
		for n in 0..100 {
			if toggle {
				assert!(n.is_even());
				assert!(!n.is_odd());
			} else {
				assert!(!n.is_even());
				assert!(n.is_odd());
			}
			toggle ^= true;
		}
	}

	#[test]
	fn test_usize() {
		assert!(1_usize.is_odd());
		assert!(2_usize.is_even());
		assert!(15_usize.is_odd());
	}
}

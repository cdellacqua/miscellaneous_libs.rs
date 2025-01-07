use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::PoisonError;

pub trait LockExt<'a, T, Guard>
where
	T: ?Sized + 'a,
{
	fn try_with_lock<O, Op: FnOnce(&T) -> O>(&'a self, op: Op) -> Option<O>;
	fn try_with_lock_mut<O, Op: FnOnce(&mut T) -> O>(&'a self, op: Op) -> Option<O>;

	///
	/// # Errors
	/// - [`PoisonError`]
	///
	fn with_lock<O, Op: FnOnce(&T) -> O>(
		&'a self,
		op: Op,
	) -> Result<O, PoisonError<MutexGuard<'a, T>>>;

	///
	/// # Errors
	/// - [`PoisonError`]
	///
	fn with_lock_mut<O, Op: FnOnce(&mut T) -> O>(
		&'a self,
		op: Op,
	) -> Result<O, PoisonError<MutexGuard<'a, T>>>;
}

impl<'a, T> LockExt<'a, T, MutexGuard<'a, T>> for Mutex<T>
where
	T: ?Sized + 'a,
{
	fn try_with_lock<O, Op: for<'b> FnOnce(&'b T) -> O>(&'a self, op: Op) -> Option<O> {
		if let Ok(guard) = self.try_lock() {
			let output = op(&guard);
			drop(guard);
			Some(output)
		} else {
			None
		}
	}
	fn try_with_lock_mut<O, Op: for<'b> FnOnce(&'b mut T) -> O>(&'a self, op: Op) -> Option<O> {
		if let Ok(mut guard) = self.try_lock() {
			let output = op(&mut guard);
			drop(guard);
			Some(output)
		} else {
			None
		}
	}

	fn with_lock<O, Op: for<'b> FnOnce(&'b T) -> O>(
		&'a self,
		op: Op,
	) -> Result<O, PoisonError<MutexGuard<'a, T>>> {
		let guard = self.lock()?;
		let output = op(&guard);
		drop(guard);
		Ok(output)
	}

	fn with_lock_mut<O, Op: for<'b> FnOnce(&'b mut T) -> O>(
		&'a self,
		op: Op,
	) -> Result<O, PoisonError<MutexGuard<'a, T>>> {
		let mut guard = self.lock()?;
		let output = op(&mut guard);
		drop(guard);
		Ok(output)
	}
}

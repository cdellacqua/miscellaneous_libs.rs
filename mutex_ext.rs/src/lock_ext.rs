use std::sync::Mutex;
use std::sync::MutexGuard;

pub trait LockExt<'a, T, Guard>
where
	T: ?Sized + 'a,
{
	fn try_with_lock<O, Op: FnOnce(&T) -> O>(&'a self, op: Op) -> Option<O>;
	fn try_with_lock_mut<O, Op: FnOnce(&mut T) -> O>(&'a self, op: Op) -> Option<O>;

	fn with_lock<O, Op: FnOnce(&T) -> O>(&'a self, op: Op) -> O;
	fn with_lock_mut<O, Op: FnOnce(&mut T) -> O>(&'a self, op: Op) -> O;
}

impl<'a, T> LockExt<'a, T, MutexGuard<'a, T>> for Mutex<T>
where
	T: ?Sized + 'a,
{
	fn try_with_lock<O, Op: FnOnce(&T) -> O>(&'a self, op: Op) -> Option<O> {
		if let Ok(guard) = self.try_lock() {
			let output = op(&guard);
			drop(guard);
			Some(output)
		} else {
			None
		}
	}
	fn try_with_lock_mut<O, Op: FnOnce(&mut T) -> O>(&'a self, op: Op) -> Option<O> {
		if let Ok(mut guard) = self.try_lock() {
			let output = op(&mut guard);
			drop(guard);
			Some(output)
		} else {
			None
		}
	}

	fn with_lock<O, Op: FnOnce(&T) -> O>(&'a self, op: Op) -> O {
		let guard = self.lock().unwrap();
		let output = op(&guard);
		drop(guard);
		output
	}

	fn with_lock_mut<O, Op: FnOnce(&mut T) -> O>(&'a self, op: Op) -> O {
		let mut guard = self.lock().unwrap();
		let output = op(&mut guard);
		drop(guard);
		output
	}
}

use std::sync::Condvar;
use std::sync::Mutex;
use std::sync::MutexGuard;

pub trait CondvarExt<'a, T, Guard> {
	fn try_with_lock<O, Op: FnOnce(&T) -> O>(&'a self, op: Op) -> Option<O>;
	fn try_with_lock_mut<O, Op: FnOnce(&mut T) -> O>(&'a self, op: Op) -> Option<O>;

	fn with_lock<O, Op: FnOnce(&T) -> O>(&'a self, op: Op) -> O;
	fn with_lock_mut<O, Op: FnOnce(&mut T) -> O>(&'a self, op: Op) -> O;

	fn wait_then<O, C: FnMut(&mut T) -> bool, Op: FnOnce(&T) -> O>(
		&'a self,
		condition: C,
		op: Op,
	) -> O;
	fn wait_then_mut<O, C: FnMut(&mut T) -> bool, Op: FnOnce(&mut T) -> O>(
		&'a self,
		condition: C,
		op: Op,
	) -> O;
}

impl<'a, T: Sized + 'a> CondvarExt<'a, T, MutexGuard<'a, T>> for (Mutex<T>, Condvar) {
	fn try_with_lock<O, Op: FnOnce(&T) -> O>(&'a self, op: Op) -> Option<O> {
		if let Ok(guard) = self.0.try_lock() {
			let output = op(&guard);
			drop(guard);
			Some(output)
		} else {
			None
		}
	}
	fn try_with_lock_mut<O, Op: FnOnce(&mut T) -> O>(&'a self, op: Op) -> Option<O> {
		if let Ok(mut guard) = self.0.try_lock() {
			let output = op(&mut guard);
			drop(guard);
			self.1.notify_all();
			Some(output)
		} else {
			None
		}
	}

	fn with_lock<O, Op: FnOnce(&T) -> O>(&'a self, op: Op) -> O {
		let guard = self.0.lock().unwrap();
		let output = op(&guard);
		drop(guard);
		output
	}

	fn with_lock_mut<O, Op: FnOnce(&mut T) -> O>(&'a self, op: Op) -> O {
		let mut guard = self.0.lock().unwrap();
		let output = op(&mut guard);
		drop(guard);
		self.1.notify_all();
		output
	}

	fn wait_then<O, C: FnMut(&mut T) -> bool, Op: FnOnce(&T) -> O>(
		&'a self,
		condition: C,
		op: Op,
	) -> O {
		let guard = self
			.1
			.wait_while(self.0.lock().unwrap(), condition)
			.unwrap();
		let output = op(&guard);
		drop(guard);
		self.1.notify_all();
		output
	}

	fn wait_then_mut<O, C: FnMut(&mut T) -> bool, Op: FnOnce(&mut T) -> O>(
		&'a self,
		condition: C,
		op: Op,
	) -> O {
		let mut guard = self
			.1
			.wait_while(self.0.lock().unwrap(), condition)
			.unwrap();
		let output = op(&mut guard);
		drop(guard);
		self.1.notify_all();
		output
	}
}

use std::sync::{Arc, Condvar, Mutex, MutexGuard};

use crate::CondvarExt;

#[derive(Debug, Clone)]
pub struct ReactiveCondvar<T>(Arc<(Mutex<T>, Condvar)>);

impl<T> ReactiveCondvar<T> {
	pub fn new(initial_value: T) -> Self {
		Self(Arc::new((Mutex::new(initial_value), Condvar::default())))
	}

	pub fn notify_all(&self) {
		self.0 .1.notify_all();
	}

	pub fn notify_one(&self) {
		self.0 .1.notify_one();
	}
}

impl<'a, T> CondvarExt<'a, T, MutexGuard<'a, T>> for ReactiveCondvar<T> {
	fn try_with_lock<O, Op: FnOnce(&T) -> O>(&'a self, op: Op) -> Option<O> {
		self.0.try_with_lock(op)
	}

	fn try_with_lock_mut<O, Op: FnOnce(&mut T) -> O>(&'a self, op: Op) -> Option<O> {
		self.0.try_with_lock_mut(op)
	}

	fn with_lock<O, Op: FnOnce(&T) -> O>(&'a self, op: Op) -> O {
		self.0.with_lock(op)
	}

	fn with_lock_mut<O, Op: FnOnce(&mut T) -> O>(&'a self, op: Op) -> O {
		self.0.with_lock_mut(op)
	}

	fn wait_while_and_then<O, C: FnMut(&mut T) -> bool, Op: FnOnce(&T) -> O>(
		&'a self,
		condition: C,
		op: Op,
	) -> O {
		self.0.wait_while_and_then(condition, op)
	}

	fn wait_while_and_then_mut<O, C: FnMut(&mut T) -> bool, Op: FnOnce(&mut T) -> O>(
		&'a self,
		condition: C,
		op: Op,
	) -> O {
		self.0.wait_while_and_then_mut(condition, op)
	}

	fn wait_timeout_while_and_then<O, C: FnMut(&mut T) -> bool, Op: FnOnce(&T) -> O>(
		&'a self,
		condition: C,
		timeout: std::time::Duration,
		op: Op,
	) -> Option<O> {
		self.0.wait_timeout_while_and_then(condition, timeout, op)
	}

	fn wait_timeout_while_and_then_mut<O, C: FnMut(&mut T) -> bool, Op: FnOnce(&mut T) -> O>(
		&'a self,
		condition: C,
		timeout: std::time::Duration,
		op: Op,
	) -> Option<O> {
		self.0
			.wait_timeout_while_and_then_mut(condition, timeout, op)
	}
}

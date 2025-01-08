#![doc = include_str!("../README.md")]

use std::{
	fmt::Debug,
	marker::PhantomData,
	sync::{Arc, Condvar, Mutex},
	thread::{self, JoinHandle},
};

use mutex_ext::LockExt;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DaemonState<QuitReason> {
	Holding,
	Quitting(Option<QuitReason>),
	Quit(Option<QuitReason>),
}

#[derive(Debug)]
pub struct ResourceDaemon<T, QuitReason: Clone + Send + 'static> {
	phantom: PhantomData<T>,
	state: Arc<(Mutex<DaemonState<QuitReason>>, Condvar)>,
	thread_handle: Option<JoinHandle<()>>,
}

#[derive(Debug, Clone)]
pub struct QuitSignal<QuitReason: Clone + Send + 'static>(
	Arc<(Mutex<DaemonState<QuitReason>>, Condvar)>,
);

impl<QuitReason: Clone + Send + 'static> QuitSignal<QuitReason> {
	pub fn dispatch(&self, reason: QuitReason) {
		wake_to_quit(&self.0, Some(reason));
	}
}

fn wake_to_quit<QuitReason: Clone + Send + 'static>(
	state: &Arc<(Mutex<DaemonState<QuitReason>>, Condvar)>,
	reason: Option<QuitReason>,
) {
	let mut guard = state.0.lock().unwrap();
	*guard = DaemonState::Quitting(reason);
	state.1.notify_one();
}

impl<T, QuitReason: Clone + Send + 'static> ResourceDaemon<T, QuitReason> {
	// Panic is actually inside the thread
	#[allow(clippy::missing_panics_doc)]
	pub fn new<
		Provider: FnOnce(QuitSignal<QuitReason>) -> Result<T, QuitReason> + Send + 'static,
	>(
		resource_provider: Provider,
	) -> Self {
		let state = Arc::new((Mutex::new(DaemonState::Holding), Condvar::default()));
		Self {
			thread_handle: Some(thread::spawn({
				let state = state.clone();
				move || {
					let resource = resource_provider({
						let state = state.clone();
						QuitSignal(state)
					});
					match resource {
						Err(err) => {
							state
								.0
								.with_lock_mut(|s| *s = DaemonState::Quit(Some(err)))
								.unwrap();
						}
						Ok(resource) => {
							let mut state = state
								.1
								.wait_while(state.0.lock().unwrap(), |q| {
									matches!(q, DaemonState::Holding)
								})
								.unwrap();
							drop(resource);

							match *state {
								DaemonState::Holding => panic!(
									"internal error: wait_while on condvar exited when predicate was false"
								),
								DaemonState::Quitting(ref mut reason) => {
									*state = DaemonState::Quit(reason.take());
								}
								DaemonState::Quit(_) => (),
							}
						}
					}
				}
			})),
			phantom: PhantomData,
			state,
		}
	}

	fn wake_to_quit_and_join(&mut self, reason: Option<QuitReason>) {
		wake_to_quit(&self.state, reason);
		if let Some(join_handle) = self.thread_handle.take() {
			join_handle.join().unwrap();
		}
	}

	///
	/// Drop the associated resource and stops the daemon thread
	///
	/// # Panics
	/// - if the mutex guarding the state of the associated thread is poisoned
	/// - if joining the associated thread fails
	pub fn give_up(&mut self, reason: QuitReason) {
		self.wake_to_quit_and_join(Some(reason));
	}

	///
	/// # Panics
	/// - if the mutex guarding the state of the associated thread is poisoned
	#[must_use]
	pub fn state(&self) -> DaemonState<QuitReason> {
		self.state.0.lock().unwrap().clone()
	}
}

impl<T, QuitReason: Clone + Send + 'static> Drop for ResourceDaemon<T, QuitReason> {
	fn drop(&mut self) {
		self.wake_to_quit_and_join(None);
	}
}

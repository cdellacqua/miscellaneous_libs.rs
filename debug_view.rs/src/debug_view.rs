#![cfg(debug_assertions)]

use macroquad::prelude::*;
use mutex_ext::{CondvarExt, ReactiveCondvar};
use std::{
	sync::{LazyLock, Mutex},
	thread::{spawn, JoinHandle},
};

pub type UICommand = dyn FnOnce() + Send + Sync + 'static;

pub struct DebugView {
	latest_command: ReactiveCondvar<Option<Box<UICommand>>>,
	window: Option<JoinHandle<()>>,
}

impl DebugView {
	#[must_use]
	pub fn new() -> Self {
		let latest_command =
			ReactiveCondvar::new(None::<Box<dyn FnOnce() + Send + Sync + 'static>>);

		Self {
			latest_command: latest_command.clone(),
			window: Some(spawn(move || {
				let amain = async move {
					loop {
						if let Some(latest_command) = latest_command
							.wait_while_and_then_mut(|cmd| cmd.is_none(), Option::take)
						{
							latest_command();
						}
						next_frame().await;
					}
				};
				macroquad::Window::new("debug_view", amain);
			})),
		}
	}

	/// # Panics
	/// - if the channel used to send commands to the UI thread is broken
	pub fn run(&mut self, command: Box<dyn FnOnce() + Send + Sync + 'static>) {
		self.latest_command
			.with_lock_mut(|latest_command| latest_command.replace(command));
	}
}

impl Default for DebugView {
	fn default() -> Self {
		Self::new()
	}
}

impl Drop for DebugView {
	fn drop(&mut self) {
		if let Some(window) = self.window.take() {
			window.join().unwrap();
		}
	}
}

pub static DEBUG_VIEW: LazyLock<Mutex<DebugView>> = LazyLock::new(|| Mutex::new(DebugView::new()));

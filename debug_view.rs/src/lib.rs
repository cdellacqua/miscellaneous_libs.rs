mod debug_view;
pub use debug_view::*;

pub use macroquad;

#[macro_export]
macro_rules! if_debug_view {
	($code:block) => {
		#[cfg(debug_assertions)]
		use debug_view::macroquad::prelude::*;
		#[cfg(debug_assertions)]
		{
			$code
		}
	};
}

#[macro_export]
macro_rules! draw_debug_view_frame {
	($code:block) => {
		debug_view::if_debug_view!({
			use mutex_ext::LockExt;
			debug_view::DEBUG_VIEW.with_lock_mut(move |debug_view| {
				use debug_view::macroquad::prelude::*;
				debug_view.run(Box::new(move || $code));
			});
		})
	};
}

# resource daemon

In the context of this library, a "resource daemon" is a long running thread which
instantiates and owns a resource, passively waiting until a request is made to drop it.

## When to use it

When you have a resource which is not Send and you need to create some simple event loop
to handle it's disposal without locking the main thread.

### Example

```rust ignore
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use resource_daemon::{ResourceDaemon, DaemonState};

let device = cpal::default_host()
	.input_devices()
	.expect("input devices to be available")
	.next()
	.expect("at least one input device");

let config = device
	.default_input_config()
	.expect("default config to be available");

let mut stream_daemon = ResourceDaemon::new({
	move |quit_signal| {
		device.build_input_stream(
			&config.into(),
			move |_: &[f32], _| {
				// ...
			},
			move |err| {
				quit_signal.dispatch(err.to_string());
			},
			None
		)
			.map_err(|err| (err.to_string()))
			.and_then(|stream| {
				stream
					.play()
					.map(|()| stream)
					.map_err(|err| (err.to_string()))
			})
	}
});

assert!(matches!(stream_daemon.state(), DaemonState::Holding));
// ...
stream_daemon.quit("cancelled by the user".to_string()); // or, equivalently, drop(stream_daemon);

assert!(matches!(stream_daemon.state(), DaemonState::Quit(_)));
```

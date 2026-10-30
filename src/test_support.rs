//! One Slint testing backend per test thread. `init_no_event_loop` sets a platform
//! per thread, and Slint panics if it is set twice on one thread, so the flag is thread-local.
//! `cargo nextest` gives each test its own process; `cargo test` may reuse a thread, which this
//! guard tolerates.

use std::cell::Cell;

use slint::private_unstable_api::re_exports::{ColorScheme, WindowInner};

thread_local! {
    static READY: Cell<bool> = const { Cell::new(false) };
}

/// Initializes the headless Slint testing backend once on the current thread.
pub(crate) fn init_platform() {
    READY.with(|ready| {
        if !ready.get() {
            i_slint_backend_testing::init_no_event_loop();
            ready.set(true);
        }
    });
}

/// Reports the OS colour scheme, as the winit backend does when Windows switches between light
/// and dark mode. The scheme is shared by every window on the thread.
pub(crate) fn set_os_dark(window: &slint::Window, dark: bool) {
    let scheme = if dark {
        ColorScheme::Dark
    } else {
        ColorScheme::Light
    };
    WindowInner::from_pub(window)
        .context()
        .set_color_scheme(scheme);
}

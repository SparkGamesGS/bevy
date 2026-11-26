//! Provides Android functionality for Bevy Engine.

#[cfg(target_os = "android")]
pub use android_activity;

/// [`AndroidApp`] provides an interface to query the application state as well as monitor events
/// (for example lifecycle and input events).
#[cfg(target_os = "android")]
pub static ANDROID_APP: std::sync::OnceLock<android_activity::AndroidApp> =
    std::sync::OnceLock::new();

/// Ends the process once the app's `main` has returned. `#[bevy_main]` calls
/// this on Android.
///
/// Bevy can't run a second time in one process: winit allows one event loop
/// per process, and [`ANDROID_APP`] keeps the first activity. Android keeps a
/// process alive after its activity finishes and may start the activity again
/// in it, which would then fail. Ending the process makes the next start a
/// fresh one.
///
/// This uses `_exit`, which skips `atexit` handlers and C++ static
/// destructors: GameActivity's call into Java from this thread with the main
/// thread's JNI handle, which Android aborts on.
#[cfg(target_os = "android")]
#[expect(
    unsafe_code,
    reason = "Ending the process without its exit handlers needs libc's `_exit`."
)]
pub fn exit_process(code: i32) -> ! {
    // SAFETY: `_exit` ends the process at once; no Rust code runs after it.
    unsafe { libc::_exit(code) }
}

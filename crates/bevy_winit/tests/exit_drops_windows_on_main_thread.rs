//! An app that exits while a render thread holds its window must not hang.
//!
//! On macOS, winit drops a window on the main thread: a drop on any other thread waits for the
//! main thread. Pipelined rendering keeps each window's [`RawHandleWrapper`] in the render world,
//! on the render thread, and when the app's world is torn down at exit, the main thread waits for
//! the render thread to hand the render world back. If the main thread has let go of the window
//! by then, the render thread drops the last reference, and each thread waits for the other.
//!
//! The renderer here is a stand-in: a thread that keeps the window's [`RawHandleWrapper`] and
//! lets it go only while the main thread waits for it during teardown, as
//! `RenderAppChannels` does. It runs on the main thread (`harness = false`) because winit's event
//! loop must, and only on macOS, where the drop waits for the main thread.

#[cfg(target_os = "macos")]
fn main() {
    use bevy_a11y::AccessibilityPlugin;
    use bevy_app::{App, AppExit, Update};
    use bevy_ecs::prelude::*;
    use bevy_input::InputPlugin;
    use bevy_window::{RawHandleWrapper, Window, WindowPlugin};
    use bevy_winit::WinitPlugin;
    use std::{
        process,
        sync::mpsc,
        thread::{self, JoinHandle},
        time::Duration,
    };

    /// The stand-in render thread.
    #[derive(Resource)]
    struct RenderThread {
        release: Option<mpsc::Sender<()>>,
        thread: Option<JoinHandle<()>>,
    }

    impl Drop for RenderThread {
        // Like `RenderAppChannels`: the main thread waits for the render thread.
        fn drop(&mut self) {
            drop(self.release.take());
            self.thread.take().unwrap().join().unwrap();
        }
    }

    // Inserted once the window exists, as the renderer is.
    fn start_render_thread(
        mut commands: Commands,
        windows: Query<&RawHandleWrapper>,
        render_thread: Option<Res<RenderThread>>,
    ) {
        if render_thread.is_some() {
            return;
        }
        let Ok(handle) = windows.single() else {
            return;
        };
        let handle = handle.clone();
        let (release, released) = mpsc::channel::<()>();
        let thread = thread::spawn(move || {
            // Returns when the sender is dropped.
            let _ = released.recv();
            drop(handle);
        });
        commands.insert_resource(RenderThread {
            release: Some(release),
            thread: Some(thread),
        });
    }

    fn exit_while_rendering(
        render_thread: Option<Res<RenderThread>>,
        mut exit: MessageWriter<AppExit>,
    ) {
        if render_thread.is_some() {
            exit.write(AppExit::Success);
        }
    }

    #[expect(clippy::print_stderr, reason = "A hung test must say why it failed.")]
    fn fail_if_hung() {
        thread::sleep(Duration::from_secs(30));
        eprintln!("the app hung on exit: the render thread dropped the window's last reference");
        process::exit(1);
    }
    thread::spawn(fail_if_hung);

    let exit = App::new()
        .add_plugins((
            InputPlugin,
            WindowPlugin {
                primary_window: Some(Window {
                    visible: false,
                    ..Default::default()
                }),
                ..Default::default()
            },
            AccessibilityPlugin,
            WinitPlugin::default(),
        ))
        .add_systems(Update, (start_render_thread, exit_while_rendering).chain())
        .run();

    assert_eq!(exit, AppExit::Success);
}

#[cfg(not(target_os = "macos"))]
fn main() {}

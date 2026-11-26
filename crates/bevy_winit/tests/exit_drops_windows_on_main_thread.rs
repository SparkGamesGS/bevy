//! An app that exits while a render thread holds its windows must not hang.
//!
//! On macOS, winit drops a window on the main thread: a drop on any other thread waits for the
//! main thread. Pipelined rendering keeps each window's [`RawHandleWrapper`] in the render world,
//! on the render thread, and when the app's world is torn down at exit, the main thread waits for
//! the render thread to hand the render world back. If the main thread has let go of a window by
//! then, the render thread drops its last reference, and each thread waits for the other.
//!
//! The renderer here is a stand-in: a thread that keeps the windows' [`RawHandleWrapper`]s and
//! lets them go only while the main thread waits for it during teardown, as `RenderAppChannels`
//! does. The app exits with one window open and one closed in that last frame. It runs on the
//! main thread (`harness = false`) because winit's event loop must, and only on macOS, where the
//! drop waits for the main thread. Its windows are hidden and unfocused.

#[cfg(target_os = "macos")]
fn main() {
    use bevy_a11y::AccessibilityPlugin;
    use bevy_app::{App, AppExit, Startup, Update};
    use bevy_ecs::prelude::*;
    use bevy_input::InputPlugin;
    use bevy_window::{PrimaryWindow, RawHandleWrapper, Window, WindowPlugin};
    use bevy_winit::WinitPlugin;
    use std::{
        process,
        sync::mpsc,
        thread::{self, JoinHandle},
        time::Duration,
    };

    fn hidden_window() -> Window {
        Window {
            visible: false,
            focused: false,
            ..Default::default()
        }
    }

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

    fn spawn_second_window(mut commands: Commands) {
        commands.spawn(hidden_window());
    }

    // Starts once both windows exist, as the renderer does, then exits in that frame with the
    // second window closed.
    fn start_render_thread_and_exit(
        mut commands: Commands,
        windows: Query<(Entity, &RawHandleWrapper, Has<PrimaryWindow>)>,
        render_thread: Option<Res<RenderThread>>,
        mut exit: MessageWriter<AppExit>,
    ) {
        if render_thread.is_some() || windows.iter().len() < 2 {
            return;
        }
        let handles: Vec<RawHandleWrapper> = windows
            .iter()
            .map(|(_, handle, _)| handle.clone())
            .collect();
        let (release, released) = mpsc::channel::<()>();
        let thread = thread::spawn(move || {
            // Returns when the sender is dropped.
            let _ = released.recv();
            drop(handles);
        });
        commands.insert_resource(RenderThread {
            release: Some(release),
            thread: Some(thread),
        });

        for (entity, _, is_primary) in &windows {
            if !is_primary {
                commands.entity(entity).despawn();
            }
        }
        exit.write(AppExit::Success);
    }

    #[expect(clippy::print_stderr, reason = "A hung test must say why it failed.")]
    fn fail_if_hung() {
        thread::sleep(Duration::from_secs(30));
        eprintln!("the app hung on exit: the render thread dropped a window's last reference");
        process::exit(1);
    }
    thread::spawn(fail_if_hung);

    let exit = App::new()
        .add_plugins((
            InputPlugin,
            WindowPlugin {
                primary_window: Some(hidden_window()),
                ..Default::default()
            },
            AccessibilityPlugin,
            WinitPlugin::default(),
        ))
        .add_systems(Startup, spawn_second_window)
        .add_systems(Update, start_render_thread_and_exit)
        .run();

    assert_eq!(exit, AppExit::Success);
}

#[cfg(not(target_os = "macos"))]
fn main() {}

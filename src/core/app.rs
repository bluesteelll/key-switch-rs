use windows::{
    Win32::{
        Foundation::{ERROR_ACCESS_DENIED, LPARAM, WIN32_ERROR, WPARAM},
        System::{Console::*, Threading::*},
        UI::WindowsAndMessaging::*,
    },
    core::*,
};

use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::core::executor::{Executor, QUEUE_CAPACITY, Stopped, Win32Runner};
use crate::data::binding::Binding;
use crate::hook::control_window::ControlWindow;
use crate::hook::ipc_server::{self, DaemonControl, DaemonStatus};
use crate::hook::keyboard_hook::KeyboardHook;
use crate::hook::main_thread::{MainThread, MainThreadHandle};
use crate::hook::watchdog::HookWatchdog;
use crate::hook::{config_watcher, keyboard_hook_callback};
use crate::ipc;
use crate::logging::counters;
use crate::platform::priority::RaisedPriority;

static MAIN_THREAD_ID: AtomicU32 = AtomicU32::new(0);

/// The action executor while it is attached to the process-wide hook.
/// Detaches and stops it when dropped, so that every return of `App::run`
/// stops it, after the hooks are removed; the normal path calls
/// [`stop`](Self::stop) right after removing them.
struct AttachedExecutor {
    hook: &'static KeyboardHook,
    executor: Option<Executor>,
}

impl AttachedExecutor {
    fn attach(hook: &'static KeyboardHook, executor: Executor) -> Self {
        hook.attach_executor(Some(executor.handle()));
        Self {
            hook,
            executor: Some(executor),
        }
    }

    /// Detaches the executor, so that a late action is `Closed`, then stops
    /// it: no queued action and no further macro step runs, and the step
    /// running now, if any, gets 2 s to finish. Main thread, after the
    /// hooks are removed: a macro's `SendInput` waits on the hook thread
    /// while that thread blocks here.
    fn stop(&mut self) {
        let Some(executor) = self.executor.take() else {
            return;
        };
        self.hook.attach_executor(None);
        if executor.stop() == Stopped::TimedOut {
            log::warn!("the action executor did not stop within 2 s; exiting without it");
        }
    }
}

impl Drop for AttachedExecutor {
    fn drop(&mut self) {
        self.stop();
    }
}

/// The daemon's side of the IPC commands, called on the IPC connection
/// workers. `on` and `off` are requests to the main thread, which owns the
/// engine, flips the remapping switch and resets the engine between two
/// events (roadmap item 2.2); `exit` posts `WM_QUIT` to the main thread,
/// which then removes the hooks itself.
struct DaemonHandle {
    hook: &'static KeyboardHook,
    main: MainThreadHandle,
}

impl DaemonControl for DaemonHandle {
    fn set_enabled(&self, enabled: bool) -> std::result::Result<(), String> {
        self.main
            .set_enabled(enabled)
            .map(drop)
            .map_err(|error| error.to_string())
    }

    fn status(&self) -> DaemonStatus {
        DaemonStatus {
            enabled: self.hook.is_enabled(),
            bindings: self.hook.bindings().len(),
        }
    }

    fn request_exit(&self) -> std::result::Result<(), String> {
        self.main.request_exit().map_err(|error| error.to_string())
    }
}

pub struct App {
    pending_bindings: Vec<Binding>,
    /// If set, install a notify-based file watcher on this path after the
    /// keyboard hook goes live, so saves to `config.ron` reload bindings
    /// without a restart. None disables hot-reload (e.g. when the user
    /// passes a config inline, or for tests).
    config_path: Option<PathBuf>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            pending_bindings: Vec::new(),
            config_path: None,
        }
    }

    /// Append a binding. Auto-blocker expansion is the responsibility of the
    /// config loader (`config::load` / `config::from_ron_str`), so the
    /// pre-expanded list can be fed back into `KeyboardHook::update_bindings`
    /// unchanged during a hot-reload.
    pub fn add_binding(mut self, binding: Binding) -> Self {
        self.pending_bindings.push(binding);
        self
    }

    /// Enable hot-reload from a config file. The watcher is started just after
    /// the keyboard hook is installed and torn down when `run` exits.
    pub fn with_config_watcher(mut self, path: PathBuf) -> Self {
        self.config_path = Some(path);
        self
    }

    /// Console only, never logged: a binding line can hold the text of a
    /// macro's `Text(...)` step. The flush puts the records queued so far,
    /// the loader's among them, above the banner.
    fn print_welcome(&self) {
        crate::logging::flush();
        println!("Key Switch started");
        println!("─────────────────────────────────────");

        for binding in &self.pending_bindings {
            println!("{}", binding);
        }

        println!("─────────────────────────────────────");
        if self.config_path.is_some() {
            println!("Edit config.ron and save — bindings reload automatically.");
        }
        println!("Press Ctrl+C to exit\n");
    }

    fn setup_ctrl_c_handler(&self) -> Result<()> {
        // SAFETY: console_ctrl_handler is a 'static extern "system" fn
        // with no captured state, so it stays valid for as long as the
        // system may call it; the call reads no other memory of ours.
        unsafe {
            SetConsoleCtrlHandler(Some(console_ctrl_handler), true)?;
        }
        Ok(())
    }

    /// Runs the daemon on the calling thread, which becomes its main
    /// thread, until `WM_QUIT`: creates the IPC pipe, starts the action
    /// executor, sets up the main thread's engine and gesture timer
    /// (`hook::main_thread`), starts the hook watchdog (roadmap item 2.4),
    /// creates the control window (roadmap item 2.3), installs the hooks,
    /// serves the pipe, runs the main loop, removes the hooks, destroys the
    /// control window and stops the watchdog and the executor.
    ///
    /// A pipe that cannot be created, an executor that cannot be started or
    /// a gesture timer that cannot be made is an error before any hook is
    /// installed, so no hook ever runs without `swch` control (KS-20), with
    /// nowhere to send `Launch`, `OpenUrl` and `Sequence` (roadmap item 2.1)
    /// or without its timers (roadmap item 2.2). If the IPC server thread
    /// cannot be started, the hooks are removed again before the error is
    /// returned, and so they are if the main loop fails. On every return the
    /// executor is stopped after the hooks are removed, waiting at most 2 s
    /// for a step that still runs.
    pub fn run(mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        // SAFETY: GetCurrentThreadId takes no arguments, touches no memory
        // of ours and cannot fail.
        let main_tid = unsafe { GetCurrentThreadId() };
        MAIN_THREAD_ID.store(main_tid, Ordering::Release);

        // Before the hooks: the usual failure, a pipe name another program
        // owns, then leaves nothing to undo.
        let listener = ipc_server::bind(ipc::PIPE_NAME)
            .map_err(|error| pipe_error_message(&ipc::pipe_path(ipc::PIPE_NAME), &error))?;

        self.print_welcome();
        self.setup_ctrl_c_handler()?;

        // Before the hooks, so that the first slow action has somewhere to
        // go and the hooks never wait for one. The hook watchdog starts
        // before them too, and both stop after the hooks are removed: the
        // main thread never waits for another thread while the keyboard
        // hook is installed and the main loop does not run, which would hold
        // up every keystroke (ruling I3 of roadmap item 2.4).
        let executor = Executor::start(Win32Runner, QUEUE_CAPACITY, &counters::COUNTERS)
            .map_err(|error| format!("could not start the action executor: {error}"))?;
        // Before the hooks, on the thread that installs them: the engine and
        // every gesture timer belong to this thread, and a timer that cannot
        // be made leaves nothing to undo.
        let main = MainThread::new()
            .map_err(|error| format!("could not create the gesture timer: {error}"))?;

        let hook = keyboard_hook_callback::get_hook();
        let bindings = std::mem::take(&mut self.pending_bindings);
        hook.update_bindings(bindings);
        let mut executor = AttachedExecutor::attach(hook, executor);
        // Before the hooks (see the executor's start): the watchdog counts
        // raw keyboard input only while the keyboard hook is installed. It
        // is no reason to stop: without it the daemon runs as before it.
        let watchdog = match HookWatchdog::start(hook.health(), main.handle(), &counters::COUNTERS)
        {
            Ok(watchdog) => Some(watchdog),
            Err(error) => {
                log::warn!(
                    "the hook watchdog is off ({error}): a keyboard hook that Windows removes is not reinstalled until the daemon restarts"
                );
                None
            }
        };
        // Before the hooks, after the executor guard, so that on every
        // return it is destroyed before the guard's stop, which waits up to
        // 2 s: a top-level window must not outlive the loop that pumps it
        // (roadmap item 2.3). It takes the session notifications and the
        // session end.
        let control_window = ControlWindow::create()
            .map_err(|error| format!("could not create the control window: {error}"))?;
        // On failure the listener is dropped on return, which closes the
        // pipe, the control window is destroyed and the executor is
        // stopped; a failed WinEvent hook has already removed the keyboard
        // hook inside `install`.
        hook.install()?;
        // Once the foreground hook is live, so that a window change after
        // this read is a foreground event (roadmap item 2.3).
        main.refresh_foreground();
        log::info!("Hook installed successfully");

        // Served only once the hooks are installed: a server started before
        // a failed install would answer `swch status` with `running` while
        // the daemon waits at its [FATAL] prompt. The IPC `exit` command
        // posts WM_QUIT to this thread, like Ctrl+C.
        let control = Arc::new(DaemonHandle {
            hook,
            main: main.handle(),
        });
        if let Err(error) = listener.serve(control) {
            // Here, on the thread that installed the hooks (KS-01).
            if let Err(error) = hook.uninstall() {
                log::warn!("could not remove the keyboard hook: {error}");
            }
            return Err(format!("could not start the IPC server thread: {error}").into());
        }
        log::info!("IPC server listening (use `swch on/off/exit`)");

        // Spawn the file watcher AFTER the hook is live, and keep it alive
        // for the duration of the message loop by binding it to a local
        // (`Drop` on the watcher stops the notify backend and tells the
        // debounce thread to exit). On error we keep running without
        // hot-reload — failing to start the watcher should not take down
        // the daemon. A reload is a request to this thread, which swaps the
        // list and resets the engine between two key events (roadmap item
        // 2.3).
        let _watcher = match self.config_path.take() {
            Some(path) => match config_watcher::spawn_watcher(path.clone(), main.handle()) {
                Ok(w) => {
                    log::info!("Watching {} for changes", path.display());
                    Some(w)
                }
                Err(e) => {
                    log::warn!(
                        "could not start config file watcher for {}: {e}",
                        path.display()
                    );
                    log::info!("continuing without hot-reload");
                    None
                }
            },
            None => None,
        };

        // Every keystroke on the desktop waits for this thread while it runs
        // the loop, so it runs at THREAD_PRIORITY_HIGHEST meanwhile (roadmap
        // item 2.2), and at normal priority again for the shutdown.
        let raised = match RaisedPriority::raise() {
            Ok(raised) => Some(raised),
            Err(error) => {
                log::warn!(
                    "could not raise the main thread's priority ({error}); it runs at normal priority"
                );
                None
            }
        };
        // This is the thread that installed both hooks, so their callbacks
        // are dispatched from this loop (KS-01), and the gesture timers fire
        // in it (roadmap item 2.2).
        let ran = main.run(hook);
        drop(raised);
        // From here a `swch on` or `off` is answered at once, "the daemon is
        // shutting down": the request queue closes with the main state.
        drop(main);

        log::info!("Removing keyboard hook...");
        hook.uninstall()?;
        // After the hooks, so that the session service and the IME teardown
        // never run while the keyboard hook waits for this thread, and before
        // the executor's stop, which may wait 2 s.
        drop(control_window);
        stop_watchdog(watchdog);
        executor.stop();

        ran.map_err(|error| format!("the main message loop failed: {error}"))?;
        Ok(())
    }
}

/// Stops the hook watchdog, if it runs, waiting at most 1 s. After the
/// hooks are removed (ruling I3): a request it sends meanwhile finds no
/// hook and installs none.
fn stop_watchdog(watchdog: Option<HookWatchdog>) {
    if let Some(watchdog) = watchdog
        && !watchdog.stop()
    {
        log::warn!("the hook watchdog did not stop within 1 s; exiting without it");
    }
}

/// The fatal message for a pipe that could not be created. A taken pipe
/// name is refused with `ERROR_ACCESS_DENIED`
/// (`FILE_FLAG_FIRST_PIPE_INSTANCE`), so that error also names who is
/// likely to own it.
fn pipe_error_message(pipe_path: &str, error: &io::Error) -> String {
    let message = format!("could not create the IPC pipe {pipe_path}: {error}");
    let taken = error
        .raw_os_error()
        .and_then(|code| u32::try_from(code).ok())
        .is_some_and(|code| WIN32_ERROR(code) == ERROR_ACCESS_DENIED);
    if taken {
        format!(
            "{message}. Another program owns that pipe name: the daemon of another \
             signed-in user (one pipe serves every session until roadmap item 4.1) \
             or a key-switch-rs started before the single-instance guard"
        )
    } else {
        message
    }
}

unsafe extern "system" fn console_ctrl_handler(ctrl_type: u32) -> BOOL {
    match ctrl_type {
        CTRL_C_EVENT | CTRL_CLOSE_EVENT => {
            log::info!("Received shutdown signal...");

            let main_thread_id = MAIN_THREAD_ID.load(Ordering::Acquire);
            if main_thread_id != 0 {
                // SAFETY: PostThreadMessageW takes the thread id and the
                // message by value and reads no memory of ours; WM_QUIT
                // carries no pointer in its parameters. `run` stored the id
                // before it registered this handler. An id that is no
                // longer valid only makes the call fail, and the handler
                // has nobody to report that to.
                unsafe {
                    let _ = PostThreadMessageW(main_thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
                }
            }

            // On this thread, which the system created for the handler: for
            // a closed console the process ends when the handler returns,
            // so the queued lines are written first, within at most 1 s of
            // the 5 s the system allows.
            crate::logging::flush();
            BOOL(1)
        }
        _ => BOOL(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::windows_actions::BindAction;
    use crate::data::key_combination::KeyCombination;
    use windows::Win32::UI::Input::KeyboardAndMouse::VK_F13;

    /// A hook of its own, leaked because `DaemonHandle` holds a `'static`
    /// one like the daemon's. No hook is installed through it.
    fn leaked_hook() -> &'static KeyboardHook {
        let hook = Box::leak(Box::new(KeyboardHook::new()));
        hook.update_bindings(vec![Binding::new(
            KeyCombination::new(VK_F13),
            BindAction::DoNothing,
        )]);
        hook
    }

    #[test]
    fn daemon_handle_switches_remapping_through_the_main_thread() {
        // Roadmap item 2.2: `on` and `off` are carried out by the thread that
        // owns the engine, which runs the daemon's loop, and `exit` ends that
        // loop. The loop thread has no hook installed.
        let hook = leaked_hook();
        let (ready_tx, ready) = std::sync::mpsc::channel();
        let looper = std::thread::spawn(move || {
            let main = MainThread::new().expect("the main thread is set up");
            let _ = ready_tx.send(main.handle());
            main.run(hook)
        });
        let main = ready
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("the loop thread is ready");
        let handle = DaemonHandle { hook, main };
        assert_eq!(
            handle.status(),
            DaemonStatus {
                enabled: true,
                bindings: 1
            }
        );

        handle.set_enabled(false).expect("the main thread answers");
        assert!(!hook.is_enabled());
        assert!(!handle.status().enabled);

        handle.set_enabled(true).expect("the main thread answers");
        assert!(handle.status().enabled);
        assert!(!hook.is_installed(), "switching remapping installs nothing");

        handle.request_exit().expect("the exit is posted");
        looper
            .join()
            .expect("the loop thread does not panic")
            .expect("the loop ends on WM_QUIT without an error");
    }

    #[test]
    fn daemon_handle_refuses_exit_without_a_main_thread() {
        let handle = DaemonHandle {
            hook: leaked_hook(),
            main: MainThreadHandle::without_a_thread(),
        };
        assert!(handle.request_exit().is_err());
        assert_eq!(
            handle.set_enabled(false),
            Err("the daemon is shutting down".to_owned())
        );
    }

    #[test]
    fn the_hook_watchdog_starts_before_the_hooks_and_stops_after_them() {
        // Ruling I3: the main thread never waits for the watchdog thread
        // while the keyboard hook is installed and the main loop does not
        // run, so the watchdog starts before `install`, as the executor
        // does, and stops after `uninstall` and after the control window
        // is dropped.
        let text = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/core/app.rs"),
        )
        .expect("read src/core/app.rs");
        let run = &text[text.find("pub fn run(").expect("App::run")..];
        let position = |code: &str| {
            run.find(code)
                .unwrap_or_else(|| panic!("App::run has `{code}`"))
        };
        let start = position("HookWatchdog::start(");
        let stop = position("stop_watchdog(watchdog)");
        assert!(start < position("ControlWindow::create()"));
        assert!(start < position("hook.install()"));
        assert!(stop > position("hook.uninstall()?"));
        assert!(stop > position("drop(control_window)"));
    }

    #[test]
    fn a_taken_pipe_name_is_reported_with_its_likely_owners() {
        let path = r"\\.\pipe\key-switch-rs.sock";
        let message = pipe_error_message(path, &io::Error::from_raw_os_error(5));
        assert!(
            message.starts_with(r"could not create the IPC pipe \\.\pipe\key-switch-rs.sock: "),
            "{message}"
        );
        assert!(message.contains("another signed-in user"), "{message}");
        assert!(
            message.contains("before the single-instance guard"),
            "{message}"
        );
    }

    #[test]
    fn other_pipe_errors_are_reported_as_they_are() {
        let path = r"\\.\pipe\key-switch-rs.sock";
        let error = io::Error::new(io::ErrorKind::InvalidInput, "bad descriptor");
        assert_eq!(
            pipe_error_message(path, &error),
            format!("could not create the IPC pipe {path}: {error}")
        );
    }
}

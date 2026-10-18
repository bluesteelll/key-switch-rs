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

use crate::data::binding::Binding;
use crate::hook::ipc_server::{self, DaemonControl, DaemonStatus};
use crate::hook::keyboard_hook::KeyboardHook;
use crate::hook::{config_watcher, keyboard_hook_callback};
use crate::ipc;

static MAIN_THREAD_ID: AtomicU32 = AtomicU32::new(0);

/// The daemon's side of the IPC commands, called on the IPC connection
/// workers. `on` and `off` only flip the remapping switch of the
/// process-wide hook; `exit` posts `WM_QUIT` to the main thread, which then
/// removes the hooks itself.
struct DaemonHandle {
    hook: &'static KeyboardHook,
    /// The thread that pumps `GetMessageW`, or 0 if unknown.
    main_thread_id: u32,
}

impl DaemonControl for DaemonHandle {
    fn set_enabled(&self, enabled: bool) -> std::result::Result<(), String> {
        self.hook
            .set_enabled(enabled)
            .map(drop)
            .ok_or_else(|| "the engine is busy on this thread".to_owned())
    }

    fn status(&self) -> DaemonStatus {
        DaemonStatus {
            enabled: self.hook.is_enabled(),
            bindings: self.hook.bindings().len(),
        }
    }

    fn request_exit(&self) -> std::result::Result<(), String> {
        if self.main_thread_id == 0 {
            return Err("main thread id unknown, cannot post WM_QUIT".to_owned());
        }
        // SAFETY: PostThreadMessageW takes the thread id and the message by
        // value and reads no memory of ours; WM_QUIT carries no pointer in
        // its parameters. A thread id that is no longer valid only makes the
        // call fail, and that error is returned.
        unsafe { PostThreadMessageW(self.main_thread_id, WM_QUIT, WPARAM(0), LPARAM(0)) }
            .map_err(|error| error.to_string())
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
    /// thread, until `WM_QUIT`: creates the IPC pipe, installs both hooks,
    /// serves the pipe, pumps messages and removes the hooks.
    ///
    /// A pipe that cannot be created is an error before any hook is
    /// installed, so no hook ever runs without `swch` control (KS-20). If
    /// the IPC server thread cannot be started, the hooks are removed again
    /// before the error is returned.
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

        let hook = keyboard_hook_callback::get_hook();
        let bindings = std::mem::take(&mut self.pending_bindings);
        hook.update_bindings(bindings);
        // On failure the listener is dropped on return, which closes the
        // pipe; a failed foreground hook has already removed the keyboard
        // hook inside `install`.
        hook.install()?;
        log::info!("Hook installed successfully");

        // Served only once the hooks are installed: a server started before
        // a failed install would answer `swch status` with `running` while
        // the daemon waits at its [FATAL] prompt. The IPC `exit` command
        // posts WM_QUIT to this thread, like Ctrl+C.
        let control = Arc::new(DaemonHandle {
            hook,
            main_thread_id: main_tid,
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
        // the daemon.
        let _watcher = match self.config_path.take() {
            Some(path) => match config_watcher::spawn_watcher(path.clone(), hook) {
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

        // SAFETY: `msg` is a local that outlives every call: GetMessageW
        // writes it, TranslateMessage and DispatchMessageW only read it.
        // This is the thread that installed both hooks, so their callbacks
        // are dispatched from this loop (KS-01).
        unsafe {
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        log::info!("Removing keyboard hook...");
        hook.uninstall()?;

        Ok(())
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
    fn daemon_handle_status_follows_the_hook_flag() {
        let hook = leaked_hook();
        let handle = DaemonHandle {
            hook,
            main_thread_id: 0,
        };
        assert_eq!(
            handle.status(),
            DaemonStatus {
                enabled: true,
                bindings: 1
            }
        );

        handle.set_enabled(false).expect("not inside with_engine");
        assert!(!hook.is_enabled());
        assert!(!handle.status().enabled);

        handle.set_enabled(true).expect("not inside with_engine");
        assert!(handle.status().enabled);
        assert!(!hook.is_installed(), "switching remapping installs nothing");
    }

    #[test]
    fn daemon_handle_refuses_exit_without_a_main_thread() {
        let handle = DaemonHandle {
            hook: leaked_hook(),
            main_thread_id: 0,
        };
        assert!(handle.request_exit().is_err());
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

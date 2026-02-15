use windows::{
    core::*,
    Win32::{
        Foundation::{LPARAM, WPARAM},
        System::{Console::*, Threading::*},
        UI::WindowsAndMessaging::*,
    },
};

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::data::binding::Binding;
use crate::hook::{config_watcher, ipc_server, keyboard_hook_callback};

static MAIN_THREAD_ID: AtomicU32 = AtomicU32::new(0);

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

    fn print_welcome(&self) {
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
        unsafe {
            SetConsoleCtrlHandler(Some(console_ctrl_handler), true)?;
        }
        Ok(())
    }

    pub fn run(mut self) -> Result<()> {
        let main_tid = unsafe { GetCurrentThreadId() };
        MAIN_THREAD_ID.store(main_tid, Ordering::Release);
        // The IPC `exit` command needs the same thread id to post WM_QUIT.
        ipc_server::set_main_thread_id(main_tid);

        self.print_welcome();
        self.setup_ctrl_c_handler()?;

        let hook = keyboard_hook_callback::get_hook();
        let bindings = std::mem::take(&mut self.pending_bindings);
        hook.update_bindings(bindings);
        hook.install()?;
        println!("✓ Hook installed successfully");

        if ipc_server::spawn(hook) {
            println!("✓ IPC server listening (use `swch on/off/exit`)\n");
        } else {
            println!();
        }

        // Spawn the file watcher AFTER the hook is live, and keep it alive
        // for the duration of the message loop by binding it to a local
        // (`Drop` on the watcher stops the notify backend and tells the
        // debounce thread to exit). On error we keep running without
        // hot-reload — failing to start the watcher should not take down
        // the daemon.
        let _watcher = match self.config_path.take() {
            Some(path) => match config_watcher::spawn_watcher(path.clone(), hook) {
                Ok(w) => {
                    println!("✓ Watching {} for changes\n", path.display());
                    Some(w)
                }
                Err(e) => {
                    eprintln!(
                        "[WARN] could not start config file watcher for {}: {}",
                        path.display(),
                        e
                    );
                    eprintln!("[INFO] continuing without hot-reload");
                    None
                }
            },
            None => None,
        };

        unsafe {
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        println!("Removing keyboard hook...");
        hook.uninstall()?;

        Ok(())
    }
}

unsafe extern "system" fn console_ctrl_handler(ctrl_type: u32) -> BOOL {
    match ctrl_type {
        CTRL_C_EVENT | CTRL_CLOSE_EVENT => {
            println!("\nReceived shutdown signal...");

            let main_thread_id = MAIN_THREAD_ID.load(Ordering::Acquire);
            if main_thread_id != 0 {
                unsafe {
                    let _ = PostThreadMessageW(
                        main_thread_id,
                        WM_QUIT,
                        WPARAM(0),
                        LPARAM(0),
                    );
                }
            }

            BOOL(1)
        }
        _ => BOOL(0),
    }
}

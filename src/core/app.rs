use windows::{
    core::*,
    Win32::{
        Foundation::{LPARAM, WPARAM},
        System::{Console::*, Threading::*},
        UI::WindowsAndMessaging::*,
    },
};

use std::sync::atomic::{AtomicU32, Ordering};

use crate::binding::Binding;
use crate::hook::keyboard_hook_callback;

static MAIN_THREAD_ID: AtomicU32 = AtomicU32::new(0);

pub struct App;

impl App {
    pub fn new() -> Self {
        Self
    }

    pub fn add_binding(self, binding: Binding) -> Self {
        let hook = keyboard_hook_callback::get_hook();

        if binding.block_original_combo {
            let system_combos = binding.action.get_system_combinations();

            for system_combo in system_combos {
                let already_exists = hook
                    .bindings()
                    .iter()
                    .any(|b| b.combination == system_combo);

                if !already_exists {
                    let blocker = Binding::new_auto_blocker(system_combo);
                    hook.add_binding(blocker);
                }
            }
        }

        hook.add_binding(binding);

        self
    }

    fn print_welcome(&self) {
        println!("Key Switch started");
        println!("─────────────────────────────────────");

        for binding in keyboard_hook_callback::get_hook().bindings() {
            println!("{}", binding);
        }

        println!("─────────────────────────────────────");
        println!("Press Ctrl+C to exit\n");
    }

    fn setup_ctrl_c_handler(&self) -> Result<()> {
        unsafe {
            SetConsoleCtrlHandler(Some(console_ctrl_handler), true)?;
        }
        Ok(())
    }

    fn install_hook(&self) -> Result<()> {
        keyboard_hook_callback::get_hook().install()?;
        println!("✓ Hook installed successfully\n");
        Ok(())
    }

    pub fn run(self) -> Result<()> {
        unsafe {
            MAIN_THREAD_ID.store(GetCurrentThreadId(), Ordering::Release);
        }

        self.print_welcome();
        self.setup_ctrl_c_handler()?;
        self.install_hook()?;

        unsafe {
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        println!("Removing keyboard hook...");
        keyboard_hook_callback::get_hook().uninstall()?;

        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        println!("\nShutting down...");
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

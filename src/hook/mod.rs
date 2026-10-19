pub mod config_watcher;
pub(crate) mod control_window;
#[cfg(debug_assertions)]
pub(crate) mod debug_hold;
pub mod health;
pub(crate) mod installer;
pub mod ipc_server;
pub mod keyboard_hook;
pub mod keyboard_hook_callback;
pub(crate) mod main_thread;
pub(crate) mod watchdog;

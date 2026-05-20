//! key-switch-rs internals exposed as a library so the `swch` CLI binary
//! can share the IPC protocol module with the daemon. The default binary
//! (`key-switch-rs.exe`) imports everything from here too.

pub mod config;
pub mod core;
pub mod data;
pub mod hook;
pub mod ipc;
pub mod system;

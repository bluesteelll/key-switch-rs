//! key-switch-rs internals exposed as a library so the `swch` CLI binary
//! can share the IPC protocol module with the daemon. The default binary
//! (`key-switch-rs.exe`) imports everything from here too.

pub mod config;
pub mod core;
pub mod data;
pub mod hook;
pub mod ipc;
pub mod system;

// Cargo never runs tests inside a build script, so the build script's logic
// is compiled into the library's test harness and tested from there. Its
// `main` is unused in that context.
#[cfg(test)]
#[allow(dead_code)]
#[path = "../build.rs"]
mod build_script;

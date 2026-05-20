//! Daemon-side IPC listener. Spawned as a background thread by `App::run`,
//! accepts connections on the named pipe, dispatches commands to the hook
//! and `App`, and writes formatted responses back.
//!
//! Cross-platform via `interprocess::local_socket` — on Windows this maps
//! to a Win32 named pipe (`\\.\pipe\<NAME>`), no network stack involved.
//!
//! Each connection handles exactly one request/response then closes. This
//! is the simplest reliable protocol for a CLI driver where commands are
//! independent — no session state to carry across requests.

use std::io::{BufRead, BufReader};
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;

use interprocess::local_socket::traits::{ListenerExt, Stream as StreamTrait};
use interprocess::local_socket::{prelude::*, GenericNamespaced, ListenerOptions};
use windows::Win32::{
    Foundation::{LPARAM, WPARAM},
    UI::WindowsAndMessaging::{PostThreadMessageW, WM_QUIT},
};

use crate::hook::keyboard_hook::KeyboardHook;
use crate::ipc::{self, format_err, format_ok, Command};

/// Thread ID of the daemon's main loop, set by `App::run` before the IPC
/// server starts. The `exit` command uses it to post `WM_QUIT` to the
/// right thread — the same mechanism Ctrl+C handling uses.
static MAIN_THREAD_ID: AtomicU32 = AtomicU32::new(0);

pub fn set_main_thread_id(id: u32) {
    MAIN_THREAD_ID.store(id, Ordering::Release);
}

/// Spawn the listener thread. Returns immediately; the thread runs for the
/// lifetime of the daemon. Bind errors are logged and the daemon continues
/// without IPC (the user just won't be able to use `swch on/off/exit`).
pub fn spawn(hook: &'static KeyboardHook) -> bool {
    let name = match ipc::PIPE_NAME.to_ns_name::<GenericNamespaced>() {
        Ok(n) => n,
        Err(e) => {
            eprintln!("[WARN] could not build pipe name: {}", e);
            return false;
        }
    };

    let listener = match ListenerOptions::new().name(name).create_sync() {
        Ok(l) => l,
        Err(e) => {
            eprintln!(
                "[WARN] could not create IPC listener at pipe {:?}: {}",
                ipc::PIPE_NAME,
                e
            );
            eprintln!(
                "[INFO] another daemon instance may already be running. \
                 `swch` commands will not reach this process."
            );
            return false;
        }
    };

    thread::Builder::new()
        .name("ipc-server".into())
        .spawn(move || {
            for conn in listener.incoming() {
                match conn {
                    Ok(stream) => {
                        // Each request is independent; spawn a short-lived
                        // worker so a slow client can't block subsequent
                        // commands.
                        thread::spawn(move || handle_connection(stream, hook));
                    }
                    Err(e) => {
                        eprintln!("[WARN] IPC accept error: {}", e);
                    }
                }
            }
        })
        .expect("spawn ipc-server thread");

    true
}

fn handle_connection<S>(stream: S, hook: &'static KeyboardHook)
where
    S: StreamTrait,
{
    // Bracket the connection in a buffered reader so we can read a single
    // line cleanly. Writes go through a separate handle obtained via the
    // shared stream's split — but the API exposes Read+Write on the same
    // object, so we just hold the original and use it for both.
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    if let Err(e) = reader.read_line(&mut line) {
        eprintln!("[WARN] IPC read error: {}", e);
        return;
    }

    let mut stream = reader.into_inner();

    let response = match Command::parse(&line) {
        Ok(cmd) => dispatch(cmd, hook),
        Err(e) => format_err(&e),
    };

    if let Err(e) = stream.write_all(response.as_bytes()) {
        eprintln!("[WARN] IPC write error: {}", e);
    }
    let _ = stream.flush();
}

fn dispatch(cmd: Command, hook: &'static KeyboardHook) -> String {
    match cmd {
        Command::On => match hook.enable() {
            Ok(()) => format_ok(if hook.is_installed() {
                "enabled"
            } else {
                "enabled (but reports not installed — investigate)"
            }),
            Err(e) => format_err(&format!("enable failed: {}", e)),
        },
        Command::Off => match hook.disable() {
            Ok(()) => format_ok("disabled"),
            Err(e) => format_err(&format!("disable failed: {}", e)),
        },
        Command::Status => {
            let state = if hook.is_installed() { "running" } else { "disabled" };
            let count = hook.bindings().len();
            format_ok(&format!("{} ({} bindings live)", state, count))
        }
        Command::Exit => {
            let main_tid = MAIN_THREAD_ID.load(Ordering::Acquire);
            if main_tid == 0 {
                return format_err("main thread id unknown — cannot post WM_QUIT");
            }
            // Post AFTER constructing the response so the client gets it
            // before the daemon tears down.
            let response = format_ok("shutting down");
            // SAFETY: PostThreadMessageW takes a valid thread id; we use
            // the one stashed by App::run from GetCurrentThreadId.
            unsafe {
                let _ = PostThreadMessageW(main_tid, WM_QUIT, WPARAM(0), LPARAM(0));
            }
            response
        }
    }
}

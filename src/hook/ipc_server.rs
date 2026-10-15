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

use std::io::{self, BufRead, BufReader};
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;

use interprocess::local_socket::traits::{ListenerExt, Stream as StreamTrait};
use interprocess::local_socket::{GenericNamespaced, Listener, ListenerOptions, prelude::*};
use interprocess::os::windows::{
    ToWtf16, local_socket::ListenerOptionsExt, security_descriptor::SecurityDescriptor,
};
use windows::Win32::{
    Foundation::{LPARAM, WPARAM},
    UI::WindowsAndMessaging::{PostThreadMessageW, WM_QUIT},
};

use crate::hook::keyboard_hook::KeyboardHook;
use crate::ipc::{self, Command, format_err, format_ok};

/// Thread ID of the daemon's main loop, set by `App::run` before the IPC
/// server starts. The `exit` command uses it to post `WM_QUIT` to the
/// right thread — the same mechanism Ctrl+C handling uses.
static MAIN_THREAD_ID: AtomicU32 = AtomicU32::new(0);

pub fn set_main_thread_id(id: u32) {
    MAIN_THREAD_ID.store(id, Ordering::Release);
}

/// The pipe's security descriptor (KS-02), in the security descriptor
/// string format. Without one, Windows gives the pipe its default DACL,
/// whose owner entry is Administrators when the daemon runs elevated; that
/// entry is deny-only in a normal console's token, so `swch` got "Access is
/// denied".
///
/// There is no `O:` (owner) or `G:` (group) part: Windows takes both from
/// the creator's token. interprocess 2.4.2 also writes the group into the
/// owner slot when it copies a descriptor for the listener's later pipe
/// instances, so a `G:` part would replace the owner.
pub const PIPE_SDDL: &str = concat!(
    // Protected DACL: nothing is inherited.
    "D:P",
    // LocalSystem: full control.
    "(A;;GA;;;SY)",
    // Administrators: full control. An elevated daemon creates every later
    // pipe instance through this entry, whatever the logon type.
    "(A;;GA;;;BA)",
    // Interactive users: read and write, so a non-elevated swch reaches an
    // elevated daemon. GW includes FILE_APPEND_DATA, which is also
    // FILE_CREATE_PIPE_INSTANCE: a daemon started from a normal console
    // creates its later instances through this entry.
    "(A;;GRGW;;;IU)",
    // Medium mandatory label: a process below medium integrity can neither
    // write to nor read from the pipe; medium and high are admitted.
    "S:(ML;;NWNR;;;ME)",
);

/// Parses [`PIPE_SDDL`].
pub fn pipe_security() -> io::Result<SecurityDescriptor> {
    security_from_sddl(PIPE_SDDL)
}

fn security_from_sddl(sddl: &str) -> io::Result<SecurityDescriptor> {
    let wide = sddl
        .to_wtf_16()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;
    SecurityDescriptor::deserialize(&wide)
}

/// Creates the listener on the namespaced pipe `pipe_name`
/// (`\\.\pipe\<pipe_name>`), protected by [`PIPE_SDDL`]. The descriptor is
/// parsed once here; the listener keeps a copy for its later instances.
fn create_listener(pipe_name: &str) -> io::Result<Listener> {
    let name = pipe_name.to_ns_name::<GenericNamespaced>()?;
    ListenerOptions::new()
        .name(name)
        .security_descriptor(pipe_security()?)
        .create_sync()
}

/// Spawn the listener thread. Returns immediately; the thread runs for the
/// lifetime of the daemon. Bind errors are logged and the daemon continues
/// without IPC (the user just won't be able to use `swch on/off/exit`).
pub fn spawn(hook: &'static KeyboardHook) -> bool {
    let listener = match create_listener(ipc::PIPE_NAME) {
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
        Command::On => match hook.set_enabled(true) {
            Some(_) => format_ok("enabled"),
            None => format_err("the engine is busy on this thread"),
        },
        Command::Off => match hook.set_enabled(false) {
            Some(_) => format_ok("disabled"),
            None => format_err("the engine is busy on this thread"),
        },
        Command::Status => {
            let state = if hook.is_enabled() {
                "running"
            } else {
                "disabled"
            };
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

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::time::Duration;

    use interprocess::local_socket::Stream;

    use super::*;

    #[test]
    fn pipe_sddl_parses() {
        pipe_security().expect("PIPE_SDDL is a valid security descriptor string");
    }

    #[test]
    fn malformed_sddl_is_rejected() {
        // Shows that `pipe_sddl_parses` can fail.
        assert!(security_from_sddl("D:P(A;;GA;;;NOT_A_SID)").is_err());
    }

    #[test]
    fn listener_with_the_pipe_security_admits_this_process() {
        // A pipe of its own, so no daemon on this machine is involved.
        let name = format!("key-switch-rs-unit-{}-sd.sock", std::process::id());
        let _listener = create_listener(&name)
            .expect("CreateNamedPipeW accepts the descriptor, its medium label included");

        // On a helper thread, so a pipe that never admits the client fails
        // the test after the timeout instead of hanging the suite.
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let connected = name
                .to_ns_name::<GenericNamespaced>()
                .and_then(Stream::connect);
            let _ = tx.send(connected.map(drop));
        });
        rx.recv_timeout(Duration::from_secs(5))
            .expect("connect did not return within 5 s")
            .expect("the DACL admits this process's token");
    }
}

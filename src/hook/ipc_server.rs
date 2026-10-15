//! Daemon-side IPC listener. Spawned as a background thread by `App::run`,
//! accepts connections on a named pipe, hands each command to a
//! [`DaemonControl`] and writes the formatted response back.
//!
//! The server knows nothing of the keyboard hook: the daemon passes its own
//! `DaemonControl` (`core::app`), and the tests pass a stand-in on a pipe
//! name of their own (`tests/ipc_roundtrip.rs`).
//!
//! Cross-platform via `interprocess::local_socket` — on Windows this maps
//! to a Win32 named pipe (`\\.\pipe\<NAME>`), no network stack involved.
//!
//! Each connection handles exactly one request/response then closes. This
//! is the simplest reliable protocol for a CLI driver where commands are
//! independent — no session state to carry across requests.

use std::io::{self, BufRead, BufReader};
use std::sync::Arc;
use std::thread;

use interprocess::local_socket::traits::{ListenerExt, Stream as StreamTrait};
use interprocess::local_socket::{GenericNamespaced, Listener, ListenerOptions, prelude::*};
use interprocess::os::windows::{
    ToWtf16, local_socket::ListenerOptionsExt, security_descriptor::SecurityDescriptor,
};

use crate::ipc::{Command, format_err, format_ok};

/// What the IPC commands do to the daemon. Called on the connection
/// workers, one per request, possibly several at once.
pub trait DaemonControl: Send + Sync {
    /// Turns remapping on or off (`on`, `off`). Asking for the position it
    /// already has is not an error.
    fn set_enabled(&self, enabled: bool) -> Result<(), String>;

    /// What `status` reports.
    fn status(&self) -> DaemonStatus;

    /// Asks the daemon to shut down (`exit`).
    fn request_exit(&self) -> Result<(), String>;
}

/// What `status` reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DaemonStatus {
    /// Whether remapping is on.
    pub enabled: bool,
    /// How many bindings are live.
    pub bindings: usize,
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

/// Creates the pipe `pipe_name` and spawns the `ipc-server` thread that
/// serves it with `control`. Returns once the pipe exists; the thread runs
/// for the lifetime of the process. An error means no pipe and no thread:
/// the caller decides whether to go on without IPC.
pub fn spawn(pipe_name: &str, control: Arc<dyn DaemonControl>) -> io::Result<()> {
    let listener = create_listener(pipe_name)?;
    thread::Builder::new()
        .name("ipc-server".into())
        .spawn(move || {
            for conn in listener.incoming() {
                match conn {
                    Ok(stream) => {
                        // Each request is independent; spawn a short-lived
                        // worker so a slow client can't block subsequent
                        // commands.
                        let control = Arc::clone(&control);
                        thread::spawn(move || handle_connection(stream, control.as_ref()));
                    }
                    Err(e) => {
                        eprintln!("[WARN] IPC accept error: {}", e);
                    }
                }
            }
        })?;
    Ok(())
}

fn handle_connection<S>(stream: S, control: &dyn DaemonControl)
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
        Ok(cmd) => dispatch(cmd, control),
        Err(e) => format_err(&e),
    };

    if let Err(e) = stream.write_all(response.as_bytes()) {
        eprintln!("[WARN] IPC write error: {}", e);
    }
    let _ = stream.flush();
}

/// Carries out one command and formats the reply line.
fn dispatch(cmd: Command, control: &dyn DaemonControl) -> String {
    match cmd {
        Command::On => match control.set_enabled(true) {
            Ok(()) => format_ok("enabled"),
            Err(e) => format_err(&format!("could not turn remapping on: {e}")),
        },
        Command::Off => match control.set_enabled(false) {
            Ok(()) => format_ok("disabled"),
            Err(e) => format_err(&format!("could not turn remapping off: {e}")),
        },
        Command::Status => {
            let DaemonStatus { enabled, bindings } = control.status();
            let state = if enabled { "running" } else { "disabled" };
            format_ok(&format!("{state} ({bindings} bindings live)"))
        }
        Command::Exit => match control.request_exit() {
            Ok(()) => format_ok("shutting down"),
            Err(e) => format_err(&format!("could not shut down: {e}")),
        },
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::sync::mpsc;
    use std::time::Duration;

    use interprocess::local_socket::Stream;

    use super::*;

    /// A stand-in for the daemon: a flag, three bindings and a count of
    /// exit requests. With `fail` every request it can refuse fails.
    struct FakeControl {
        enabled: AtomicBool,
        fail: bool,
        exits: AtomicU32,
    }

    impl FakeControl {
        fn new(fail: bool) -> Self {
            Self {
                enabled: AtomicBool::new(true),
                fail,
                exits: AtomicU32::new(0),
            }
        }
    }

    impl DaemonControl for FakeControl {
        fn set_enabled(&self, enabled: bool) -> Result<(), String> {
            if self.fail {
                return Err("the fake refuses".to_owned());
            }
            self.enabled.store(enabled, Ordering::Release);
            Ok(())
        }

        fn status(&self) -> DaemonStatus {
            DaemonStatus {
                enabled: self.enabled.load(Ordering::Acquire),
                bindings: 3,
            }
        }

        fn request_exit(&self) -> Result<(), String> {
            if self.fail {
                return Err("the fake refuses".to_owned());
            }
            self.exits.fetch_add(1, Ordering::AcqRel);
            Ok(())
        }
    }

    #[test]
    fn dispatch_switches_remapping_and_status_reports_the_flag() {
        let control = FakeControl::new(false);
        assert_eq!(
            dispatch(Command::Status, &control),
            "OK: running (3 bindings live)\n"
        );
        assert_eq!(dispatch(Command::Off, &control), "OK: disabled\n");
        assert_eq!(
            dispatch(Command::Status, &control),
            "OK: disabled (3 bindings live)\n"
        );
        assert_eq!(
            dispatch(Command::Off, &control),
            "OK: disabled\n",
            "already off is not an error"
        );
        assert_eq!(dispatch(Command::On, &control), "OK: enabled\n");
        assert_eq!(
            dispatch(Command::Status, &control),
            "OK: running (3 bindings live)\n"
        );
    }

    #[test]
    fn dispatch_reports_a_control_failure_as_err() {
        let control = FakeControl::new(true);
        for command in [Command::On, Command::Exit] {
            let reply = dispatch(command.clone(), &control);
            assert!(reply.starts_with("ERR: "), "{command:?}: {reply:?}");
            assert!(reply.contains("the fake refuses"), "{command:?}: {reply:?}");
        }
    }

    #[test]
    fn dispatch_exit_asks_the_daemon_to_quit_once() {
        let control = FakeControl::new(false);
        assert_eq!(dispatch(Command::Exit, &control), "OK: shutting down\n");
        assert_eq!(control.exits.load(Ordering::Acquire), 1);
    }

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
    fn pipe_sddl_grants_exactly_system_admins_and_interactive_users_and_labels_medium() {
        // No test can read the label back from a created pipe (manual check
        // M2), and `listener_with_the_pipe_security_admits_this_process`
        // passes with no label at all. This pins the descriptor's shape, so
        // that dropping the label or widening the DACL turns a test red.
        let (dacl, sacl) = PIPE_SDDL
            .split_once("S:")
            .expect("PIPE_SDDL has a SACL part that carries the mandatory label");
        assert_eq!(
            sacl, "(ML;;NWNR;;;ME)",
            "the only SACL entry is the medium mandatory label (no write up, no read up)"
        );
        assert!(
            dacl.starts_with("D:P"),
            "the DACL is protected, so nothing is inherited"
        );
        for ace in ["(A;;GA;;;SY)", "(A;;GA;;;BA)", "(A;;GRGW;;;IU)"] {
            assert!(dacl.contains(ace), "the DACL grants {ace}");
        }
        assert_eq!(
            dacl.matches('(').count(),
            3,
            "SYSTEM, Administrators and interactive users are the only DACL entries"
        );
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

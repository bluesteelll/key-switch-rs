//! IPC protocol between the daemon (`key-switch-rs.exe`) and the CLI
//! (`swch.exe`). Wire format is **one ASCII line per command** + one ASCII
//! line per response — newline-terminated, no length prefix, no framing.
//!
//! ```text
//! client → daemon: "on\n"
//! daemon → client: "OK: enabled\n"
//! ```
//!
//! Transport is a Windows named pipe via the `interprocess` crate's
//! cross-platform `local_socket` API (on Windows this is a named pipe;
//! on Unix it would be a Unix domain socket).
//!
//! The client side is [`request`], which `swch` uses: it connects through
//! interprocess's named-pipe API, which can bound the wait for a busy pipe,
//! and gives up once its deadline ([`REPLY_TIMEOUT`] for `swch`) has passed
//! without a reply, because interprocess pipe streams take no I/O timeouts.

use std::fmt;
use std::io::{self, BufRead, BufReader, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use interprocess::ConnectWaitMode;
use interprocess::os::windows::named_pipe::{DuplexPipeStream, pipe_mode};
use windows::Win32::Foundation::{
    ERROR_ACCESS_DENIED, ERROR_FILE_NOT_FOUND, ERROR_PIPE_BUSY, ERROR_SEM_TIMEOUT, WIN32_ERROR,
};

/// Local-socket name used by both sides. On Windows the OS-mapped form
/// resolves to `\\.\pipe\key-switch-rs.sock`.
pub const PIPE_NAME: &str = "key-switch-rs.sock";

/// How long `swch` waits for the daemon, from the start of the connect to
/// the end of the reply.
pub const REPLY_TIMEOUT: Duration = Duration::from_secs(5);

/// The path of the namespaced pipe `pipe_name`: interprocess maps a
/// `GenericNamespaced` name to `\\.\pipe\<pipe_name>` the same way.
pub fn pipe_path(pipe_name: &str) -> String {
    format!(r"\\.\pipe\{pipe_name}")
}

/// Why [`request`] got no reply.
#[derive(Debug)]
pub enum RequestError {
    /// No pipe of that name exists: no daemon is listening.
    NotRunning,
    /// The pipe exists, but its security descriptor refuses this process.
    AccessDenied,
    /// The pipe exists, but no instance became free in time.
    Busy,
    /// No reply came within the deadline.
    TimedOut(Duration),
    /// Any other I/O error.
    Io(io::Error),
}

impl RequestError {
    /// What the error says about a daemon on the pipe: `Some(false)` if
    /// none runs, `Some(true)` if one does (it refuses this process, is busy
    /// or does not answer), `None` if the error does not tell.
    pub fn daemon_running(&self) -> Option<bool> {
        match self {
            Self::NotRunning => Some(false),
            Self::AccessDenied | Self::Busy | Self::TimedOut(_) => Some(true),
            Self::Io(_) => None,
        }
    }
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRunning => f.write_str("no daemon is listening on the pipe"),
            Self::AccessDenied => f.write_str("access to the daemon's pipe is denied"),
            Self::Busy => f.write_str("the daemon's pipe is busy"),
            Self::TimedOut(timeout) => write!(f, "the daemon did not answer within {timeout:?}"),
            Self::Io(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for RequestError {}

/// Sends `command` to the daemon on the namespaced pipe `pipe_name` and
/// returns its parsed reply ([`parse_response`]). Connect, send and reply
/// share one deadline, `timeout`.
///
/// The exchange runs on its own `ipc-request` thread, because a pipe read
/// cannot be given a timeout. If the deadline passes first, that thread is
/// left blocked until the server closes the pipe or the process exits.
pub fn request(
    pipe_name: &str,
    command: &Command,
    timeout: Duration,
) -> Result<(bool, String), RequestError> {
    let path = pipe_path(pipe_name);
    let wire = command.as_wire();
    let (tx, rx) = mpsc::sync_channel(1);
    thread::Builder::new()
        .name("ipc-request".into())
        .spawn(move || {
            // Fails only if the caller gave up already; nobody waits then.
            let _ = tx.send(exchange(&path, wire, timeout));
        })
        .map_err(RequestError::Io)?;
    match rx.recv_timeout(timeout) {
        Ok(reply) => reply.map(|line| parse_response(&line)),
        Err(mpsc::RecvTimeoutError::Timeout) => Err(RequestError::TimedOut(timeout)),
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(RequestError::Io(io::Error::other(
            "the request thread ended without a result",
        ))),
    }
}

/// Connects to the pipe at `path`, waiting at most `timeout` for a free
/// instance, sends `wire` and reads one reply line.
fn exchange(path: &str, wire: &'static str, timeout: Duration) -> Result<String, RequestError> {
    let stream = DuplexPipeStream::<pipe_mode::Bytes>::connect_by_path_with_wait_mode(
        path,
        ConnectWaitMode::Timeout(timeout),
    )
    .map_err(classify_connect_error)?;
    // No flush: on a pipe it waits until the daemon has read everything,
    // and the reply shows that it has.
    (&stream)
        .write_all(wire.as_bytes())
        .map_err(RequestError::Io)?;
    let mut reply = String::new();
    BufReader::new(&stream)
        .read_line(&mut reply)
        .map_err(RequestError::Io)?;
    Ok(reply)
}

/// Sorts a connect error into what it says about the daemon.
fn classify_connect_error(error: io::Error) -> RequestError {
    let code = error
        .raw_os_error()
        .and_then(|code| u32::try_from(code).ok())
        .map(WIN32_ERROR);
    match code {
        Some(ERROR_FILE_NOT_FOUND) => RequestError::NotRunning,
        Some(ERROR_ACCESS_DENIED) => RequestError::AccessDenied,
        // WaitNamedPipeW fails with ERROR_SEM_TIMEOUT when no instance
        // became free within the wait.
        Some(ERROR_PIPE_BUSY | ERROR_SEM_TIMEOUT) => RequestError::Busy,
        // interprocess reports a pipe still busy at the end of the wait as
        // TimedOut.
        _ if error.kind() == io::ErrorKind::TimedOut => RequestError::Busy,
        _ => RequestError::Io(error),
    }
}

/// Command sent by the CLI to the daemon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Turn remapping on. The hooks stay as they are. Idempotent: already
    /// on is OK.
    On,
    /// Turn remapping off: the hooks stay as they are and pass every key on
    /// untouched, and the daemon keeps running. Idempotent.
    Off,
    /// Shut the daemon down cleanly (`PostThreadMessageW(WM_QUIT)`).
    Exit,
    /// Health check / "is daemon up". Daemon answers with a short status
    /// string (running/disabled).
    Status,
}

impl Command {
    /// Parses a line received over the wire. Lines are trimmed before
    /// matching so terminal newlines / whitespace don't matter.
    pub fn parse(line: &str) -> Result<Self, String> {
        match line.trim().to_ascii_lowercase().as_str() {
            "on" => Ok(Command::On),
            "off" => Ok(Command::Off),
            "exit" | "quit" | "shutdown" => Ok(Command::Exit),
            "status" | "ping" => Ok(Command::Status),
            other => Err(format!("unknown command {:?}", other)),
        }
    }

    /// Wire-format for sending. Always newline-terminated.
    pub fn as_wire(&self) -> &'static str {
        match self {
            Command::On => "on\n",
            Command::Off => "off\n",
            Command::Exit => "exit\n",
            Command::Status => "status\n",
        }
    }
}

/// Response framing used by the daemon. `"OK: <msg>"` for success,
/// `"ERR: <msg>"` for failure. Newline-terminated on the wire.
pub fn format_ok(msg: &str) -> String {
    format!("OK: {}\n", msg)
}

pub fn format_err(msg: &str) -> String {
    format!("ERR: {}\n", msg)
}

/// Parse a daemon response — strips OK:/ERR: prefix, returns the message
/// and whether it indicated success.
pub fn parse_response(line: &str) -> (bool, String) {
    let trimmed = line.trim();
    if let Some(rest) = trimmed.strip_prefix("OK:") {
        (true, rest.trim().to_string())
    } else if let Some(rest) = trimmed.strip_prefix("ERR:") {
        (false, rest.trim().to_string())
    } else {
        // No prefix — treat as an opaque message; success unknown but assume ok.
        (true, trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_parse_roundtrip() {
        for cmd in [Command::On, Command::Off, Command::Exit, Command::Status] {
            assert_eq!(Command::parse(cmd.as_wire()).unwrap(), cmd);
        }
    }

    #[test]
    fn command_parse_case_insensitive() {
        assert_eq!(Command::parse("ON").unwrap(), Command::On);
        assert_eq!(Command::parse("  OfF\n").unwrap(), Command::Off);
        assert_eq!(Command::parse("EXIT").unwrap(), Command::Exit);
    }

    #[test]
    fn command_parse_aliases() {
        assert_eq!(Command::parse("quit").unwrap(), Command::Exit);
        assert_eq!(Command::parse("shutdown").unwrap(), Command::Exit);
        assert_eq!(Command::parse("ping").unwrap(), Command::Status);
    }

    #[test]
    fn command_parse_unknown() {
        let err = Command::parse("hello").unwrap_err();
        assert!(err.contains("hello"));
    }

    #[test]
    fn response_parse_ok() {
        let (ok, msg) = parse_response("OK: enabled\n");
        assert!(ok);
        assert_eq!(msg, "enabled");
    }

    #[test]
    fn response_parse_err() {
        let (ok, msg) = parse_response("ERR: not running");
        assert!(!ok);
        assert_eq!(msg, "not running");
    }

    #[test]
    fn response_format() {
        assert_eq!(format_ok("done"), "OK: done\n");
        assert_eq!(format_err("oops"), "ERR: oops\n");
    }

    #[test]
    fn connect_errors_are_classified() {
        let classify = |code: i32| classify_connect_error(io::Error::from_raw_os_error(code));
        assert!(
            matches!(classify(2), RequestError::NotRunning),
            "file not found"
        );
        assert!(
            matches!(classify(5), RequestError::AccessDenied),
            "access denied"
        );
        assert!(matches!(classify(231), RequestError::Busy), "pipe busy");
        assert!(
            matches!(classify(121), RequestError::Busy),
            "semaphore timeout"
        );
        assert!(
            matches!(
                classify_connect_error(io::Error::from(io::ErrorKind::TimedOut)),
                RequestError::Busy
            ),
            "interprocess reports a pipe still busy after the wait as TimedOut"
        );
        assert!(
            matches!(classify(87), RequestError::Io(_)),
            "invalid parameter"
        );
    }

    #[test]
    fn only_a_missing_pipe_means_no_daemon() {
        assert_eq!(RequestError::NotRunning.daemon_running(), Some(false));
        assert_eq!(RequestError::AccessDenied.daemon_running(), Some(true));
        assert_eq!(RequestError::Busy.daemon_running(), Some(true));
        assert_eq!(
            RequestError::TimedOut(REPLY_TIMEOUT).daemon_running(),
            Some(true)
        );
        assert_eq!(
            RequestError::Io(io::Error::from_raw_os_error(87)).daemon_running(),
            None
        );
    }

    #[test]
    fn pipe_path_is_the_local_pipe_namespace() {
        assert_eq!(
            pipe_path("key-switch-rs.sock"),
            r"\\.\pipe\key-switch-rs.sock"
        );
    }

    #[test]
    fn timed_out_names_the_deadline() {
        assert_eq!(
            RequestError::TimedOut(Duration::from_secs(5)).to_string(),
            "the daemon did not answer within 5s"
        );
    }
}

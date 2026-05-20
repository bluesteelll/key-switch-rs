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

/// Local-socket name used by both sides. On Windows the OS-mapped form
/// resolves to `\\.\pipe\key-switch-rs.sock`.
pub const PIPE_NAME: &str = "key-switch-rs.sock";

/// Command sent by the CLI to the daemon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Enable the keyboard hook. Idempotent — already-on is OK.
    On,
    /// Disable the keyboard hook (uninstalls `WH_KEYBOARD_LL` but keeps
    /// the daemon process alive). Idempotent.
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
}

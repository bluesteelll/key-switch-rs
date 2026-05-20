//! `swch` — CLI driver for the key-switch-rs daemon.
//!
//! Subcommands:
//!
//! | Command       | What it does                                                |
//! | ------------- | ----------------------------------------------------------- |
//! | `swch open`   | Launch the daemon (`key-switch-rs.exe`) elevated via UAC.   |
//! | `swch on`     | Tell the running daemon to enable its keyboard hook.        |
//! | `swch off`    | Tell the running daemon to disable its keyboard hook.       |
//! | `swch exit`   | Tell the running daemon to shut down.                       |
//! | `swch status` | Ask the daemon for its current state (running/disabled +    |
//! |               | binding count). Useful for scripts.                         |
//!
//! All commands except `open` talk to the daemon over a named pipe.
//! If the daemon isn't running, they print a one-liner and exit 1.

use std::io::{BufRead, BufReader, Write};
use std::process::ExitCode;
use std::time::Duration;

use interprocess::local_socket::traits::Stream as StreamTrait;
use interprocess::local_socket::{prelude::*, GenericNamespaced, Stream};

use key_switch_rs::ipc::{parse_response, Command, PIPE_NAME};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_usage(&args[0]);
        return ExitCode::FAILURE;
    }

    let subcommand = args[1].to_ascii_lowercase();

    match subcommand.as_str() {
        "open" | "start" | "launch" => match cmd_open() {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("swch: {}", e);
                ExitCode::FAILURE
            }
        },
        "on" => send(Command::On),
        "off" => send(Command::Off),
        "exit" | "quit" | "shutdown" | "stop" => send(Command::Exit),
        "status" | "ping" => send(Command::Status),
        "help" | "--help" | "-h" => {
            print_usage(&args[0]);
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("swch: unknown command {:?}", other);
            print_usage(&args[0]);
            ExitCode::FAILURE
        }
    }
}

fn print_usage(prog: &str) {
    eprintln!("usage: {} <command>", prog);
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  open    Launch the daemon (UAC prompt)");
    eprintln!("  on      Enable the keyboard hook (daemon must be running)");
    eprintln!("  off     Disable the keyboard hook");
    eprintln!("  status  Show daemon state and binding count");
    eprintln!("  exit    Shut the daemon down");
}

// ---- `open` ----

fn cmd_open() -> Result<(), String> {
    // If we can already talk to the daemon, it's already running — bail
    // before triggering an unnecessary UAC prompt.
    if connect().is_ok() {
        println!("swch: daemon is already running");
        return Ok(());
    }

    // Daemon lives next to us in the install directory.
    let mut exe_path = std::env::current_exe()
        .map_err(|e| format!("could not locate current_exe: {}", e))?;
    exe_path.pop();
    exe_path.push("key-switch-rs.exe");
    if !exe_path.exists() {
        return Err(format!(
            "daemon binary not found at {}",
            exe_path.display()
        ));
    }

    // Use ShellExecuteW with the "runas" verb to trigger UAC. The daemon's
    // embedded manifest demands admin anyway, so a plain Command::spawn
    // would just fail with "operation requires elevation" (os error 740)
    // when launched from a medium-integrity console.
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let file_wide: Vec<u16> = exe_path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let verb_wide: Vec<u16> = "runas".encode_utf16().chain(std::iter::once(0)).collect();

    // SAFETY: both wide buffers are null-terminated and live for the call.
    let hinst = unsafe {
        ShellExecuteW(
            None,
            PCWSTR(verb_wide.as_ptr()),
            PCWSTR(file_wide.as_ptr()),
            None,
            None,
            SW_SHOWNORMAL,
        )
    };
    let val = hinst.0 as usize;
    if val <= 32 {
        return Err(format!(
            "ShellExecuteW failed (code {}) — user may have declined the UAC prompt",
            val
        ));
    }

    println!("swch: daemon launched");
    Ok(())
}

// ---- pipe-talking subcommands ----

fn send(cmd: Command) -> ExitCode {
    let stream = match connect() {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "swch: cannot reach daemon ({}). Use `swch open` to start it.",
                e
            );
            return ExitCode::FAILURE;
        }
    };

    let (ok, msg) = match round_trip(stream, &cmd) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("swch: IPC error: {}", e);
            return ExitCode::FAILURE;
        }
    };

    if ok {
        println!("{}", msg);
        ExitCode::SUCCESS
    } else {
        eprintln!("swch: {}", msg);
        ExitCode::FAILURE
    }
}

fn connect() -> Result<Stream, String> {
    let name = PIPE_NAME
        .to_ns_name::<GenericNamespaced>()
        .map_err(|e| format!("could not build pipe name: {}", e))?;
    Stream::connect(name).map_err(|e| format!("{}", e))
}

fn round_trip(mut stream: Stream, cmd: &Command) -> Result<(bool, String), String> {
    stream
        .write_all(cmd.as_wire().as_bytes())
        .map_err(|e| format!("write: {}", e))?;
    stream.flush().ok();

    // Set a coarse read timeout via shutdown? `interprocess` Stream
    // doesn't expose set_read_timeout cross-platform. We rely on the
    // daemon responding promptly; if it hangs, the user can Ctrl+C the
    // CLI.
    let _ = Duration::from_secs(5);

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader
        .read_line(&mut response)
        .map_err(|e| format!("read: {}", e))?;

    Ok(parse_response(&response))
}

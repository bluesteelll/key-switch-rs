//! `swch` — CLI driver for the key-switch-rs daemon.
//!
//! Subcommands:
//!
//! | Command       | What it does                                                |
//! | ------------- | ----------------------------------------------------------- |
//! | `swch open`   | Launch the daemon (`key-switch-rs.exe`) elevated via UAC,   |
//! |               | unless a daemon already runs.                               |
//! | `swch on`     | Turn remapping on.                                          |
//! | `swch off`    | Turn remapping off (keys pass through).                     |
//! | `swch exit`   | Tell the running daemon to shut down.                       |
//! | `swch status` | Ask the daemon for its current state (running/disabled +    |
//! |               | binding count). Useful for scripts.                         |
//!
//! Every command, `open` included, sends one request over the daemon's
//! named pipe (`ipc::request`) and waits at most `ipc::REPLY_TIMEOUT` (5 s)
//! for the connection and the reply. If the daemon isn't running, they
//! print a one-liner and exit 1; `open` then starts it.

use std::process::ExitCode;

use key_switch_rs::ipc::{Command, PIPE_NAME, REPLY_TIMEOUT, RequestError, request};

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
    eprintln!("  on      Turn remapping on (daemon must be running)");
    eprintln!("  off     Turn remapping off (keys pass through)");
    eprintln!("  status  Show daemon state and binding count");
    eprintln!("  exit    Shut the daemon down");
}

// ---- `open` ----

fn cmd_open() -> Result<(), String> {
    // Only a missing pipe proves that no daemon runs. A pipe that refuses
    // this console, stays busy or does not answer belongs to a running
    // daemon, and a second one would only bring another UAC prompt and a
    // daemon without a pipe (KS-20). A `status` request, not a bare
    // connect, so the daemon logs no read error for the probe (KS-39).
    match request(PIPE_NAME, &Command::Status, REPLY_TIMEOUT) {
        Ok(_) => {
            println!("swch: daemon is already running");
            return Ok(());
        }
        Err(error) => match error.daemon_running() {
            Some(true) => {
                println!("swch: daemon is already running ({error})");
                return Ok(());
            }
            Some(false) => {}
            None => {
                return Err(format!(
                    "cannot tell whether a daemon is running ({error}); not starting another"
                ));
            }
        },
    }

    // The daemon binary sits next to swch.exe.
    let mut exe_path =
        std::env::current_exe().map_err(|e| format!("could not locate current_exe: {}", e))?;
    exe_path.pop();
    exe_path.push("key-switch-rs.exe");
    if !exe_path.exists() {
        return Err(format!("daemon binary not found at {}", exe_path.display()));
    }

    // Use ShellExecuteW with the "runas" verb to trigger UAC. The daemon's
    // embedded manifest demands admin anyway, so a plain Command::spawn
    // would just fail with "operation requires elevation" (os error 740)
    // when launched from a medium-integrity console.
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    use windows::core::PCWSTR;

    let file_wide: Vec<u16> = exe_path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
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
    match request(PIPE_NAME, &cmd, REPLY_TIMEOUT) {
        Ok((true, msg)) => {
            println!("{msg}");
            ExitCode::SUCCESS
        }
        Ok((false, msg)) => {
            eprintln!("swch: {msg}");
            ExitCode::FAILURE
        }
        Err(RequestError::NotRunning) => {
            eprintln!("swch: the daemon is not running. Use `swch open` to start it.");
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!("swch: cannot reach daemon ({error}).");
            ExitCode::FAILURE
        }
    }
}

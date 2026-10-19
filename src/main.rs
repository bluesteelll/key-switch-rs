use std::io::{self, Write};
use std::process::ExitCode;

use key_switch_rs::config;
use key_switch_rs::core::app::App;
use key_switch_rs::logging;
use key_switch_rs::system::known_folder;
use key_switch_rs::system::single_instance::{INSTANCE_MUTEX_NAME, InstanceError, InstanceGuard};

/// Exit code of a start that found another daemon in this session (roadmap
/// item 1.5); a failure exits with 1.
const EXIT_ALREADY_RUNNING: u8 = 2;

fn main() -> ExitCode {
    // The logger before anything else, with the console as its only output:
    // the log file is attached in `run`, once this process owns the session,
    // so a refused second start never writes to, or rotates, the file of the
    // daemon that runs (KS-20).
    logging::init();
    // First of all: no config is read or template written, and no hook or
    // pipe exists, until this process is known to be the session's only
    // daemon (KS-20). This binary has no test harness (KS-38), so the
    // already-running path below is checked by hand only: a second start
    // prints one line and exits with 2, without the Enter prompt.
    let instance = match InstanceGuard::acquire(INSTANCE_MUTEX_NAME) {
        Ok(instance) => instance,
        Err(InstanceError::AlreadyRunning) => {
            // No prompt: a second start from a script or a Scheduled Task
            // must not leave a console waiting on stdin, and the running
            // daemon's console already shows that one runs.
            log::error!("{}", InstanceError::AlreadyRunning);
            logging::flush();
            return ExitCode::from(EXIT_ALREADY_RUNNING);
        }
        Err(error) => return fatal(&error),
    };
    match run(instance) {
        Ok(()) => {
            logging::flush();
            ExitCode::SUCCESS
        }
        Err(e) => fatal(&*e),
    }
}

/// Logs `error` as fatal, waits for Enter and returns the failure code.
fn fatal(error: &dyn std::error::Error) -> ExitCode {
    log::error!(target: logging::FATAL_TARGET, "{error}");
    // The prompt must come after the [FATAL] line. A writer that does not
    // get there within the flush timeout is stuck, so the line then goes to
    // the console from here as well.
    if !logging::flush() {
        let _ = writeln!(io::stderr(), "[FATAL] {error}");
    }
    // When launched via double-click the console window closes the
    // moment main() returns. Block on a keypress so the user can
    // actually read the error. The prompt is console UI, not a record.
    let _ = writeln!(io::stderr().lock(), "\nPress Enter to exit...");
    let mut buf = String::new();
    let _ = std::io::stdin().read_line(&mut buf);
    ExitCode::FAILURE
}

/// Takes the guard by value, so it lives until `run` returns, as every
/// parameter does whatever its pattern: after `App::run` has removed the
/// hooks, and before a fatal error waits for Enter, so that a corrected
/// restart is not refused.
fn run(_instance: InstanceGuard) -> Result<(), Box<dyn std::error::Error>> {
    attach_log_file();

    // Resolution order for the config path:
    //   1. First positional CLI argument, if provided, made absolute against
    //      the current folder. The file must exist; nothing is created.
    //   2. <exe_dir>/config.ron (portable layout: drop the .exe anywhere and
    //      its config lives beside it), written from the template if missing.
    let config = config::ConfigPath::from_arg(std::env::args_os().nth(1))?;

    let bindings = config::load(&config)?;

    let mut app = App::new().with_config_watcher(config.into_path());
    for b in bindings {
        app = app.add_binding(b);
    }
    app.run()?;
    Ok(())
}

/// Adds `%LOCALAPPDATA%\key-switch-rs\daemon.log` to the log, now that this
/// process owns the session, and logs the start line into it. Without a
/// LocalAppData folder or a log writer, the daemon logs to the console only.
fn attach_log_file() {
    let local_app_data = match known_folder::local_app_data() {
        Ok(folder) => folder,
        Err(error) => {
            log::warn!("no log file: the LocalAppData folder is unknown: {error}");
            return;
        }
    };
    let dir = logging::log_dir(&local_app_data);
    match logging::attach_file(&dir) {
        Ok(path) => log::info!(
            "key-switch-rs {} started (pid {}), logging to {}",
            env!("CARGO_PKG_VERSION"),
            std::process::id(),
            path.display()
        ),
        Err(error) => log::warn!("no log file in {}: {error}", dir.display()),
    }
}

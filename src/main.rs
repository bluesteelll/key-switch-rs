use std::path::PathBuf;
use std::process::ExitCode;

use key_switch_rs::config;
use key_switch_rs::core::app::App;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("[FATAL] {e}");
            // When launched via double-click the console window closes the
            // moment main() returns. Block on a keypress so the user can
            // actually read the error.
            eprintln!("\nPress Enter to exit...");
            let mut buf = String::new();
            let _ = std::io::stdin().read_line(&mut buf);
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Resolution order for the config path:
    //   1. First positional CLI argument, if provided.
    //   2. <exe_dir>/config.ron (portable layout: drop the .exe anywhere and
    //      its config lives beside it).
    let config_path: PathBuf = match std::env::args_os().nth(1) {
        Some(arg) => PathBuf::from(arg),
        None => config::default_config_path()?,
    };

    let bindings = config::load(&config_path)?;

    let mut app = App::new().with_config_watcher(config_path);
    for b in bindings {
        app = app.add_binding(b);
    }
    app.run()?;
    Ok(())
}

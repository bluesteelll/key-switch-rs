//! File-watcher driven hot-reload for `config.ron`.
//!
//! Lifecycle:
//!   1. Caller (`App::run`) starts the watcher after the keyboard hook is
//!      installed; the returned `RecommendedWatcher` is kept alive for the
//!      duration of the program.
//!   2. `notify` delivers filesystem events to an internal channel.
//!   3. A worker thread debounces those events (editors fire several per
//!      save) and reloads the config, atomically swapping the binding list
//!      via `KeyboardHook::update_bindings`.
//!   4. Parse errors during reload are logged; the previously-running
//!      binding list stays live, so a broken save never bricks the daemon.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::thread;
use std::time::Duration;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::config;
use crate::hook::keyboard_hook::KeyboardHook;

/// Coalescing window: any save burst that completes within this many ms is
/// reduced to a single reload. 250 ms covers VS Code / Notepad++ / vim save
/// patterns (write-temp, rename, set permissions, fsync) without being long
/// enough for a user to notice the delay between Ctrl+S and the new bindings
/// going live.
const DEBOUNCE_MS: u64 = 250;

/// The config file as the watcher sees it.
struct WatchTarget {
    path: PathBuf,
}

impl WatchTarget {
    /// `None` when `config_path` has no parent folder (`C:\`).
    fn new(config_path: &Path) -> Option<Self> {
        config_path.parent()?;
        Some(Self {
            path: config_path.to_path_buf(),
        })
    }

    /// The folder to watch, non-recursively.
    fn dir(&self) -> &Path {
        self.path
            .parent()
            .expect("invariant: WatchTarget::new checked that the path has a parent")
    }

    /// A create, modify (data, metadata, rename) or remove of the config
    /// file. Access events are left out: atime updates from our own read of
    /// the file would otherwise loop us.
    fn wants_reload(&self, event: &Event) -> bool {
        matches!(
            event.kind,
            EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_),
        ) && event.paths.iter().any(|p| p == &self.path)
    }
}

/// Install a watcher on the parent directory of `config_path` and spawn the
/// debounce + reload worker. Returns the watcher handle — caller MUST keep it
/// alive (drop = stop watching, debounce thread exits cleanly).
pub fn spawn_watcher(
    config_path: PathBuf,
    hook: &'static KeyboardHook,
) -> notify::Result<RecommendedWatcher> {
    let target = WatchTarget::new(&config_path).ok_or_else(|| {
        notify::Error::generic("the config path has no parent folder or file name")
            .add_path(config_path.clone())
    })?;
    let dir = target.dir().to_path_buf();
    let (tx, rx) = channel::<()>();

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        let Ok(event) = res else { return };

        if target.wants_reload(&event) {
            // Ignore send errors: the receiver only goes away when the
            // worker thread exits, in which case there's nothing to do.
            let _ = tx.send(());
        }
    })?;

    // Watch the parent directory, not the file itself: editors that save by
    // writing a temp file and renaming it (VS Code, vim, Sublime) delete the
    // watched inode, after which a file-level watcher stops receiving
    // events. A non-recursive directory watch is robust to that.
    watcher.watch(&dir, RecursiveMode::NonRecursive)?;

    thread::Builder::new()
        .name("config-watcher".into())
        .spawn(move || debounce_loop(rx, config_path, hook))
        .expect("spawn config-watcher thread");

    Ok(watcher)
}

fn debounce_loop(rx: Receiver<()>, config_path: PathBuf, hook: &'static KeyboardHook) {
    let debounce = Duration::from_millis(DEBOUNCE_MS);

    loop {
        // Block until first event of a burst.
        if rx.recv().is_err() {
            // Sender dropped (watcher gone) — graceful shutdown.
            return;
        }

        // Drain extra events arriving inside the debounce window. Reset the
        // timer on each one so a sequence of fast-fire events still
        // produces one reload, not many.
        loop {
            match rx.recv_timeout(debounce) {
                Ok(()) => continue,
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }

        reload_into(&config_path, hook);
    }
}

/// Loads the config at `config_path` into `hook`. On an error the bindings
/// already live stay live.
fn reload_into(config_path: &Path, hook: &KeyboardHook) {
    match config::reload(config_path) {
        Ok(new_bindings) => {
            let count = new_bindings.len();
            hook.update_bindings(new_bindings);
            println!("[INFO] config reloaded ({} bindings live)", count);
        }
        Err(e) => {
            eprintln!("[ERROR] config reload failed:\n{}", e);
            eprintln!("[INFO] keeping previous bindings live");
        }
    }
}

//! File-watcher driven hot-reload for `config.ron`.
//!
//! Lifecycle:
//!   1. Caller (`App::run`) starts the watcher after the keyboard hook is
//!      installed; the returned `RecommendedWatcher` is kept alive for the
//!      duration of the program.
//!   2. `notify` delivers the filesystem events of the config's folder to
//!      an internal channel. An event counts when the file name it reports
//!      equals the config's, compared the way the file system compares
//!      names (ordinal, ignoring case with the OS uppercase table). The
//!      watch is non-recursive, so the name alone identifies the entry; the
//!      folder part, which notify builds from the watched path, is not
//!      compared, so its letter case or a relative form cannot hide a save
//!      (KS-21).
//!   3. A worker thread debounces those events (editors fire several per
//!      save) and reloads the config, atomically swapping the binding list
//!      via `KeyboardHook::update_bindings`.
//!   4. Errors during reload are logged and the previously-running
//!      binding list stays live, so a broken save never bricks the daemon.
//!      That includes a deleted or renamed file: a reload never writes the
//!      file, so it is not recreated from the template (KS-23).

use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::thread;
use std::time::Duration;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use windows::Win32::Globalization::{CSTR_EQUAL, CompareStringOrdinal};

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
    dir: PathBuf,
    /// The config's file name in UTF-16, as `CompareStringOrdinal` takes it.
    name: Vec<u16>,
}

impl WatchTarget {
    /// `None` when `config_path` has no parent folder or no file name (`C:\`).
    fn new(config_path: &Path) -> Option<Self> {
        Some(Self {
            dir: config_path.parent()?.to_path_buf(),
            name: config_path.file_name()?.encode_wide().collect(),
        })
    }

    /// The folder to watch, non-recursively.
    fn dir(&self) -> &Path {
        &self.dir
    }

    /// A create, modify (data, metadata, rename) or remove of an entry whose
    /// file name is the config's. Access events are left out: atime updates
    /// from our own read of the file would otherwise loop us.
    fn wants_reload(&self, event: &Event) -> bool {
        matches!(
            event.kind,
            EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_),
        ) && event.paths.iter().any(|path| {
            path.file_name().is_some_and(|name| {
                same_file_name(&name.encode_wide().collect::<Vec<u16>>(), &self.name)
            })
        })
    }
}

/// Whether two file names are equal for the file system: ordinal, case
/// folded with the OS uppercase table. The `windows` wrapper converts the
/// lengths to `i32` with a panic on overflow, which a Windows path, at most
/// 32767 UTF-16 units, cannot reach.
fn same_file_name(a: &[u16], b: &[u16]) -> bool {
    if a.is_empty() || b.is_empty() {
        return false;
    }
    // SAFETY: CompareStringOrdinal reads a.len() and b.len() UTF-16 units
    // from the two slices, which are borrowed for the whole call; it writes
    // nothing and keeps no pointer. A failure returns 0, which is not
    // CSTR_EQUAL, so it counts as different names.
    unsafe { CompareStringOrdinal(a, b, true) == CSTR_EQUAL }
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::Instant;

    use notify::event::{AccessKind, CreateKind, ModifyKind, RemoveKind, RenameMode};
    use windows::Win32::UI::Input::KeyboardAndMouse::VK_F13;

    use super::*;
    use crate::config::testing::{ONE, TWO, TempDir};
    use crate::data::key_spec::KeySpec;

    fn cwd() -> PathBuf {
        std::env::current_dir().expect("the test has a current folder")
    }

    fn event(kind: EventKind, path: PathBuf) -> Event {
        Event::new(kind).add_path(path)
    }

    /// A hook of its own for the watcher thread, which needs a `'static`
    /// one. No hook is installed through it.
    fn leaked_hook() -> &'static KeyboardHook {
        Box::leak(Box::new(KeyboardHook::new()))
    }

    /// Writes `text` to `file` once a second, for up to 10 s, until `hook`
    /// has `count` bindings live; polls every 50 ms. Writing more than once
    /// covers a first write that lands before the watch is armed.
    fn rewrite_until_live(file: &Path, text: &str, hook: &KeyboardHook, count: usize) -> bool {
        let start = Instant::now();
        let mut written: Option<Instant> = None;
        while start.elapsed() < Duration::from_secs(10) {
            if written.is_none_or(|at| at.elapsed() >= Duration::from_secs(1)) {
                fs::write(file, text).expect("write the config");
                written = Some(Instant::now());
            }
            if hook.bindings().len() == count {
                return true;
            }
            thread::sleep(Duration::from_millis(50));
        }
        false
    }

    #[test]
    fn an_event_for_a_relative_config_path_matches() {
        let target = WatchTarget::new(Path::new("my.ron")).expect("has a parent");
        // notify reports the watched folder, made absolute, joined with the
        // name the file system reports.
        let modify = event(EventKind::Modify(ModifyKind::Any), cwd().join("my.ron"));
        assert!(
            target.wants_reload(&modify),
            "a Modify of <cwd>\\my.ron must reload the config given as my.ron (KS-21)"
        );
    }

    #[test]
    fn an_event_matches_the_config_file_name_in_any_case() {
        let target = WatchTarget::new(&cwd().join("my.ron")).expect("has a parent");
        for e in [
            event(EventKind::Modify(ModifyKind::Any), cwd().join("MY.RON")),
            event(EventKind::Create(CreateKind::Any), cwd().join("My.Ron")),
            event(EventKind::Remove(RemoveKind::Any), cwd().join("my.ron")),
            event(
                EventKind::Modify(ModifyKind::Name(RenameMode::To)),
                cwd().join("MY.RON"),
            ),
        ] {
            assert!(target.wants_reload(&e), "{e:?}");
        }
    }

    #[test]
    fn other_files_and_other_event_kinds_do_not_match() {
        let target = WatchTarget::new(&cwd().join("my.ron")).expect("has a parent");
        for e in [
            event(EventKind::Modify(ModifyKind::Any), cwd().join("my.ron.tmp")),
            event(EventKind::Create(CreateKind::Any), cwd().join("other.ron")),
            event(EventKind::Access(AccessKind::Read), cwd().join("my.ron")),
            event(EventKind::Any, cwd().join("my.ron")),
        ] {
            assert!(!target.wants_reload(&e), "{e:?}");
        }
    }

    #[test]
    fn same_file_name_ignores_case_like_the_file_system() {
        let utf16 = |s: &str| s.encode_utf16().collect::<Vec<u16>>();
        for (a, b) in [
            ("config.ron", "config.ron"),
            ("config.ron", "CONFIG.RON"),
            (
                "\u{43a}\u{43e}\u{43d}\u{444}\u{438}\u{433}.ron",
                "\u{41a}\u{41e}\u{41d}\u{424}\u{418}\u{413}.RON",
            ),
        ] {
            assert!(same_file_name(&utf16(a), &utf16(b)), "{a} = {b}");
        }
        for (a, b) in [
            ("config.ron", "config.ron.bak"),
            ("config.ron", "konfig.ron"),
            ("config.ron", ""),
        ] {
            assert!(!same_file_name(&utf16(a), &utf16(b)), "{a} != {b}");
        }
    }

    #[test]
    fn a_root_path_has_no_watch_target() {
        assert!(WatchTarget::new(Path::new(r"C:\")).is_none());
    }

    #[test]
    fn a_spawned_watcher_reloads_a_path_whose_case_differs_from_the_file() {
        let dir = TempDir::new("case");
        let file = dir.path().join("config.ron");
        fs::write(&file, ONE).expect("write the config");
        let hook = leaked_hook();
        hook.update_bindings(config::reload(&file).expect("the config loads"));
        // Declared after `dir`, so the watch stops before the folder goes.
        let _watcher = spawn_watcher(dir.path().join("CONFIG.RON"), hook).expect("watcher starts");

        let reloaded = rewrite_until_live(&file, TWO, hook, 2);
        assert!(
            reloaded,
            "no reload within 10 s: the watcher of CONFIG.RON never swapped in the 2 bindings of config.ron"
        );
    }

    #[test]
    fn a_deleted_config_keeps_the_live_bindings() {
        let dir = TempDir::new("deleted");
        let path = dir.path().join("config.ron");
        fs::write(&path, ONE).expect("write the config");
        let hook = KeyboardHook::new();
        hook.update_bindings(config::reload(&path).expect("the config loads"));

        fs::remove_file(&path).expect("delete the config");
        reload_into(&path, &hook);

        assert!(!path.exists(), "a deleted config must not be recreated");
        let bindings = hook.bindings();
        assert_eq!(
            bindings.len(),
            1,
            "the bindings of the deleted file stay live"
        );
        assert_eq!(
            bindings[0].combination().expect("a keys binding").keys,
            vec![KeySpec::Key(VK_F13)]
        );
    }

    #[test]
    fn a_save_with_a_misspelled_field_keeps_the_live_bindings() {
        let dir = TempDir::new("misspelled");
        let path = dir.path().join("config.ron");
        fs::write(&path, ONE).expect("write the config");
        let hook = KeyboardHook::new();
        hook.update_bindings(config::reload(&path).expect("the config loads"));

        fs::write(
            &path,
            r#"( bindngs: [ ( keys: "F14", action: DoNothing ) ] )"#,
        )
        .expect("write the misspelled config");
        reload_into(&path, &hook);

        assert_eq!(
            hook.bindings().len(),
            1,
            "a misspelled `bindings` must keep the live bindings (KS-22)"
        );
    }

    #[test]
    fn a_spawned_watcher_keeps_the_bindings_when_the_file_is_deleted() {
        let dir = TempDir::new("deleted-live");
        let path = dir.path().join("config.ron");
        fs::write(&path, ONE).expect("write the config");
        let hook = leaked_hook();
        hook.update_bindings(config::reload(&path).expect("the config loads"));
        // Declared after `dir`, so the watch stops before the folder goes.
        let _watcher = spawn_watcher(path.clone(), hook).expect("watcher starts");
        // A watcher that never reloads would keep the bindings too.
        assert!(
            rewrite_until_live(&path, TWO, hook, 2),
            "the watcher never reloaded"
        );

        fs::remove_file(&path).expect("delete the config");
        // Six times the debounce: the reload after the delete has run.
        thread::sleep(Duration::from_millis(6 * DEBOUNCE_MS));

        assert!(!path.exists(), "a deleted config must not be recreated");
        assert_eq!(
            hook.bindings().len(),
            2,
            "the bindings of the deleted file stay live"
        );
    }
}

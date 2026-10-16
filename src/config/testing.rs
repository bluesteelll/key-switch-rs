//! Test-only helpers for the loader and watcher tests that use the disk.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// A config with one binding, on F13.
pub(crate) const ONE: &str = r#"( bindings: [ ( keys: "F13", action: DoNothing ) ] )"#;

/// A config with two bindings, on F13 and F14.
pub(crate) const TWO: &str = r#"( bindings: [
    ( keys: "F13", action: DoNothing ),
    ( keys: "F14", action: DoNothing ),
] )"#;

/// A new empty folder under `std::env::temp_dir()`, removed (best effort)
/// on drop.
pub(crate) struct TempDir(PathBuf);

impl TempDir {
    /// `<temp>\key-switch-rs-test-<pid>-<n>-<label>`, `n` counting the
    /// folders of this process.
    pub(crate) fn new(label: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        // Relaxed: only the uniqueness of the returned value matters, an
        // RMW is atomic under any ordering, and no other data is published
        // through the counter.
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "key-switch-rs-test-{}-{n}-{label}",
            std::process::id()
        ));
        // A folder an earlier process with the same id left behind would
        // hand the test its files.
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create the test folder");
        Self(path)
    }

    pub(crate) fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

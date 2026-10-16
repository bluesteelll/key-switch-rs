//! One daemon per Windows session (roadmap item 1.5, KS-20).
//!
//! The guard is a named mutex that is never waited on or owned: only its
//! existence matters. The object lives while any handle to it is open, and
//! the system closes the daemon's handle when the process ends, a crash
//! included, so a later start finds the name free again.

use std::fmt;

use windows::{
    Win32::{
        Foundation::{
            CloseHandle, ERROR_ACCESS_DENIED, ERROR_ALREADY_EXISTS, ERROR_SUCCESS, GetLastError,
            HANDLE, SetLastError,
        },
        System::Threading::CreateMutexW,
    },
    core::PCWSTR,
};

/// The daemon's instance mutex, one per Windows session (roadmap item 1.5,
/// KS-20). `Local\` is the session namespace: a low-level hook sees only
/// its own session's input, so daemons of other sessions do not count.
pub const INSTANCE_MUTEX_NAME: &str = r"Local\key-switch-rs";

/// An open handle to the instance mutex. While it lives, a second
/// [`acquire`](Self::acquire) of the same name in this session fails with
/// [`InstanceError::AlreadyRunning`].
#[derive(Debug)]
#[must_use = "the instance is released when the guard is dropped"]
pub struct InstanceGuard {
    handle: HANDLE,
}

impl InstanceGuard {
    /// Creates the named mutex `name`, or reports that another holder
    /// exists.
    pub fn acquire(name: &str) -> Result<InstanceGuard, InstanceError> {
        let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
        // CreateMutexW is not documented to clear the last error when it
        // creates the mutex, so a stale ERROR_ALREADY_EXISTS from an earlier
        // call would read as another holder.
        //
        // SAFETY: SetLastError only writes this thread's last-error slot.
        unsafe { SetLastError(ERROR_SUCCESS) };
        // SAFETY: `wide` is NUL-terminated and alive across the call, which
        // only reads it. `None` gives the mutex the default security
        // descriptor and a handle that child processes cannot inherit, so a
        // `Launch` child never keeps the name alive (KS-09).
        let created = unsafe { CreateMutexW(None, false, PCWSTR(wide.as_ptr())) };
        // SAFETY: GetLastError only reads this thread's last-error slot. It
        // is read right after CreateMutexW, with no call in between.
        let last_error = unsafe { GetLastError() };
        let guard = InstanceGuard {
            handle: created.map_err(classify_open_error)?,
        };
        if last_error == ERROR_ALREADY_EXISTS {
            // Closed before returning: a refused start that kept its handle
            // would keep the name alive after the holder exits.
            drop(guard);
            return Err(InstanceError::AlreadyRunning);
        }
        Ok(guard)
    }
}

impl Drop for InstanceGuard {
    fn drop(&mut self) {
        // SAFETY: `handle` is the result of a successful CreateMutexW, owned
        // by this guard alone and closed only here, once.
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}

/// Why [`InstanceGuard::acquire`] gave no guard.
#[derive(Debug)]
pub enum InstanceError {
    /// Another holder exists: a daemon of this session, possibly at a
    /// higher integrity level that this process may not open.
    AlreadyRunning,
    /// `CreateMutexW` failed for another reason.
    Os(windows::core::Error),
}

impl fmt::Display for InstanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstanceError::AlreadyRunning => f.write_str(
                "already running in this session; use `swch status`, `swch off` or `swch exit` to control it",
            ),
            InstanceError::Os(error) => {
                write!(f, "could not create the single-instance mutex: {error}")
            }
        }
    }
}

impl std::error::Error for InstanceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            InstanceError::AlreadyRunning => None,
            InstanceError::Os(error) => Some(error),
        }
    }
}

/// Sorts a failed `CreateMutexW`. On an existing mutex it asks for
/// `MUTEX_ALL_ACCESS`, and a mutex this process may not open refuses that
/// with `ERROR_ACCESS_DENIED`, as one created by an elevated daemon is
/// expected to refuse a non-elevated start: the name is held. Any other
/// error, such as `ERROR_INVALID_HANDLE` for another kind of object under
/// that name, is the system's.
fn classify_open_error(error: windows::core::Error) -> InstanceError {
    if error.code() == ERROR_ACCESS_DENIED.to_hresult() {
        InstanceError::AlreadyRunning
    } else {
        InstanceError::Os(error)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};

    use interprocess::os::windows::ToWtf16;
    use interprocess::os::windows::security_descriptor::{
        AsSecurityDescriptor, SecurityDescriptor,
    };
    use windows::Win32::Foundation::{ERROR_INVALID_HANDLE, ERROR_INVALID_NAME};
    use windows::Win32::Security::SECURITY_ATTRIBUTES;

    use super::*;

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    /// A mutex name in this session that no other test, test run or daemon
    /// uses. No test acquires `INSTANCE_MUTEX_NAME`.
    fn unique_name(tag: &str) -> String {
        // Relaxed is enough: the counter only hands out distinct numbers and
        // publishes no other data.
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        format!(r"Local\key-switch-rs-test-{}-{tag}-{n}", std::process::id())
    }

    #[test]
    fn a_second_acquisition_of_a_held_name_is_already_running() {
        let name = unique_name("held");
        let _first = InstanceGuard::acquire(&name).expect("a fresh name is free");
        assert!(
            matches!(
                InstanceGuard::acquire(&name),
                Err(InstanceError::AlreadyRunning)
            ),
            "a second acquisition of a held name must be AlreadyRunning"
        );
    }

    #[test]
    fn the_name_is_free_again_once_every_guard_is_dropped() {
        let name = unique_name("released");
        let first = InstanceGuard::acquire(&name).expect("a fresh name is free");
        assert!(
            matches!(
                InstanceGuard::acquire(&name),
                Err(InstanceError::AlreadyRunning)
            ),
            "a second acquisition of a held name must be AlreadyRunning"
        );
        drop(first);
        let _again = InstanceGuard::acquire(&name)
            .expect("the refused acquisition must not keep the mutex alive");
    }

    #[test]
    fn distinct_names_do_not_conflict() {
        let _a = InstanceGuard::acquire(&unique_name("a")).expect("a fresh name is free");
        let _b = InstanceGuard::acquire(&unique_name("b"))
            .expect("another name is free while the first is held");
    }

    #[test]
    fn a_mutex_this_process_may_not_open_counts_as_already_running() {
        // A squatter that grants LocalSystem alone: opening it asks for
        // MUTEX_ALL_ACCESS, which this process's token does not get. An
        // elevated daemon's mutex refuses a medium-integrity start the same
        // way (manual check M3).
        let name = unique_name("denied");
        let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
        let sddl = "D:P(A;;GA;;;SY)"
            .to_wtf_16()
            .expect("the SDDL has no interior nul");
        let only_system = SecurityDescriptor::deserialize(&sddl).expect("the SDDL parses");
        let attributes = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: only_system.as_sd().cast_mut(),
            bInheritHandle: false.into(),
        };
        // SAFETY: `wide` is NUL-terminated, and `attributes` and the
        // descriptor it points to are locals of this frame; all three
        // outlive the call, which only reads them.
        let handle = unsafe { CreateMutexW(Some(&attributes), false, PCWSTR(wide.as_ptr())) }
            .expect("the creator of a new mutex gets a handle whatever its DACL");
        let _squatter = InstanceGuard { handle };

        assert!(
            matches!(
                InstanceGuard::acquire(&name),
                Err(InstanceError::AlreadyRunning)
            ),
            "a held name this process may not open must be AlreadyRunning"
        );
    }

    #[test]
    fn only_access_denied_among_open_errors_means_already_running() {
        assert!(
            matches!(
                classify_open_error(ERROR_ACCESS_DENIED.into()),
                InstanceError::AlreadyRunning
            ),
            "ERROR_ACCESS_DENIED: the name is held by a mutex this process may not open"
        );
        for code in [ERROR_INVALID_HANDLE, ERROR_INVALID_NAME] {
            match classify_open_error(code.into()) {
                InstanceError::Os(error) => assert_eq!(error.code(), code.to_hresult()),
                InstanceError::AlreadyRunning => {
                    panic!("{code:?} is an OS error, not another daemon")
                }
            }
        }
    }

    #[test]
    fn the_daemon_mutex_is_local_key_switch_rs() {
        assert_eq!(INSTANCE_MUTEX_NAME, r"Local\key-switch-rs");
    }

    #[test]
    fn a_stale_already_exists_error_does_not_make_a_fresh_name_look_held() {
        // SAFETY: SetLastError only writes this thread's last-error slot.
        unsafe { SetLastError(ERROR_ALREADY_EXISTS) };
        let _guard = InstanceGuard::acquire(&unique_name("stale"))
            .expect("a fresh name is free whatever the thread's last error was");
    }

    #[test]
    fn already_running_names_the_swch_commands() {
        assert_eq!(
            InstanceError::AlreadyRunning.to_string(),
            "already running in this session; use `swch status`, `swch off` or `swch exit` to control it"
        );
        let os = InstanceError::Os(ERROR_INVALID_HANDLE.into()).to_string();
        assert!(
            os.starts_with("could not create the single-instance mutex: "),
            "{os}"
        );
    }
}

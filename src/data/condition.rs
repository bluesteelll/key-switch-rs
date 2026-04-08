//! Foreground-context predicates used to gate bindings.
//!
//! A `Condition` decides whether a binding fires given the currently focused
//! window's executable name and title. The hook callback collects that
//! context once per keystroke (only when at least one binding actually has a
//! non-trivial condition — `Condition::Always` short-circuits) and reuses it
//! for every binding match in that callback.
//!
//! All string comparisons are case-insensitive. Windows treats exe and path
//! names case-insensitively, and humans don't reliably remember the casing
//! of window titles either.

use std::ffi::c_void;

use windows::Win32::{
    Foundation::*,
    System::{
        ProcessStatus::GetModuleBaseNameW,
        Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION},
    },
    UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId},
};

#[derive(Debug, Clone, Default)]
pub enum Condition {
    /// Always true — the binding fires whenever the combo matches. Default
    /// when the config doesn't specify `when:`.
    #[default]
    Always,
    /// True when the focused window's process is `<exe>.exe` (case
    /// insensitive). Compares against the executable's base name, so
    /// `"code.exe"` matches `C:\...\Microsoft VS Code\Code.exe`.
    AppEquals(String),
    /// True when the focused window's title contains the substring (case
    /// insensitive). Useful for picking out an app by tab name even when
    /// many windows share the same executable.
    TitleContains(String),
    /// True when the focused window's title is exactly the given string
    /// (case insensitive).
    TitleEquals(String),
    /// Negation.
    Not(Box<Condition>),
    /// Conjunction — all sub-conditions must hold.
    And(Vec<Condition>),
    /// Disjunction — at least one sub-condition must hold.
    Or(Vec<Condition>),
}

impl Condition {
    /// Cheap probe: is this condition guaranteed to skip the foreground
    /// lookup? Used by the hook callback to avoid Win32 calls when no
    /// binding cares about the focused window.
    pub fn is_always(&self) -> bool {
        matches!(self, Condition::Always)
    }

    pub fn evaluate(&self, ctx: &ForegroundContext) -> bool {
        match self {
            Condition::Always => true,
            Condition::AppEquals(target) => {
                match ctx.app() {
                    Some(actual) => actual.eq_ignore_ascii_case(target),
                    None => false,
                }
            }
            Condition::TitleContains(needle) => match ctx.title() {
                Some(actual) => actual.to_lowercase().contains(&needle.to_lowercase()),
                None => false,
            },
            Condition::TitleEquals(target) => match ctx.title() {
                Some(actual) => actual.eq_ignore_ascii_case(target),
                None => false,
            },
            Condition::Not(inner) => !inner.evaluate(ctx),
            Condition::And(parts) => parts.iter().all(|c| c.evaluate(ctx)),
            Condition::Or(parts) => parts.iter().any(|c| c.evaluate(ctx)),
        }
    }
}

/// One-shot snapshot of the focused window's identifying info. Populated
/// lazily — `app()` and `title()` each issue their Win32 calls on first use,
/// then cache the result so a callback with N condition-bearing bindings
/// pays for the lookups exactly once.
pub struct ForegroundContext {
    hwnd: HWND,
    app: std::cell::OnceCell<Option<String>>,
    title: std::cell::OnceCell<Option<String>>,
}

impl ForegroundContext {
    /// Capture the foreground window. Safe to call any time — never returns
    /// an invalid context, but `app()` / `title()` will yield `None` if the
    /// OS reports no foreground window (e.g. during secure-desktop transitions).
    pub fn capture() -> Self {
        // SAFETY: GetForegroundWindow has no preconditions.
        let hwnd = unsafe { GetForegroundWindow() };
        Self {
            hwnd,
            app: std::cell::OnceCell::new(),
            title: std::cell::OnceCell::new(),
        }
    }

    /// Base name of the focused window's executable (e.g. `"chrome.exe"`),
    /// or `None` if the window/process is gone or inaccessible.
    pub fn app(&self) -> Option<&str> {
        self.app
            .get_or_init(|| self.fetch_app())
            .as_deref()
    }

    /// Title bar text of the focused window, or `None` if the window is
    /// gone or has no title (rare).
    pub fn title(&self) -> Option<&str> {
        self.title
            .get_or_init(|| self.fetch_title())
            .as_deref()
    }

    fn fetch_app(&self) -> Option<String> {
        if self.hwnd.is_invalid() {
            return None;
        }
        // SAFETY: hwnd is the result of GetForegroundWindow and validated
        // non-invalid above. pid out-pointer is a valid stack local.
        let mut pid: u32 = 0;
        let _thread_id = unsafe { GetWindowThreadProcessId(self.hwnd, Some(&mut pid)) };
        if pid == 0 {
            return None;
        }

        // PROCESS_QUERY_LIMITED_INFORMATION is enough for GetModuleBaseNameW
        // and is permitted against most processes including elevated ones,
        // which PROCESS_QUERY_INFORMATION would deny.
        //
        // SAFETY: OpenProcess returns an owned handle that we close on drop
        // via the RAII wrapper below.
        let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?;
        let _guard = ProcessHandleGuard(handle);

        let mut buf = [0u16; 260]; // MAX_PATH; exe names are short.
        // SAFETY: handle is valid; buf is sized and writable.
        let len = unsafe { GetModuleBaseNameW(handle, None, &mut buf) };
        if len == 0 {
            return None;
        }
        Some(String::from_utf16_lossy(&buf[..len as usize]))
    }

    fn fetch_title(&self) -> Option<String> {
        if self.hwnd.is_invalid() {
            return None;
        }
        let mut buf = [0u16; 512];
        // SAFETY: hwnd validated above; buf sized and writable.
        let len = unsafe { GetWindowTextW(self.hwnd, &mut buf) };
        if len == 0 {
            return None;
        }
        Some(String::from_utf16_lossy(&buf[..len as usize]))
    }
}

/// Closes the process handle when dropped. Local RAII because the Win32
/// `CloseHandle` requires the raw handle and panics from inside `fetch_app`
/// would otherwise leak.
struct ProcessHandleGuard(HANDLE);

impl Drop for ProcessHandleGuard {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            // SAFETY: handle came from OpenProcess above and hasn't been
            // closed by anyone else.
            unsafe {
                let _ = windows::Win32::Foundation::CloseHandle(self.0);
            }
        }
    }
}

// Silence "unused" on the c_void import which is pulled in transitively by
// the windows crate's ProcessHandleGuard semantics but not directly named.
const _: Option<*mut c_void> = None;

#[cfg(test)]
mod tests {
    use super::*;

    /// `Always` short-circuits and never touches the context.
    #[test]
    fn always_is_true() {
        let ctx = ForegroundContext::capture();
        assert!(Condition::Always.evaluate(&ctx));
    }

    #[test]
    fn is_always_helper() {
        assert!(Condition::Always.is_always());
        assert!(!Condition::AppEquals("x".into()).is_always());
        assert!(!Condition::Not(Box::new(Condition::Always)).is_always());
    }

    #[test]
    fn not_inverts() {
        let ctx = ForegroundContext::capture();
        let always_true = Condition::Always;
        let always_false = Condition::Not(Box::new(always_true.clone()));
        assert!(always_true.evaluate(&ctx));
        assert!(!always_false.evaluate(&ctx));
    }

    #[test]
    fn and_short_circuits_on_first_false() {
        let ctx = ForegroundContext::capture();
        let cond = Condition::And(vec![
            Condition::Not(Box::new(Condition::Always)), // false
            Condition::Always,                            // not evaluated, but must not panic
        ]);
        assert!(!cond.evaluate(&ctx));
    }

    #[test]
    fn or_short_circuits_on_first_true() {
        let ctx = ForegroundContext::capture();
        let cond = Condition::Or(vec![
            Condition::Always,
            Condition::Not(Box::new(Condition::Always)),
        ]);
        assert!(cond.evaluate(&ctx));
    }

    #[test]
    fn nested_and_or_evaluates_correctly() {
        let ctx = ForegroundContext::capture();
        // (true AND (false OR true)) = true
        let cond = Condition::And(vec![
            Condition::Always,
            Condition::Or(vec![
                Condition::Not(Box::new(Condition::Always)),
                Condition::Always,
            ]),
        ]);
        assert!(cond.evaluate(&ctx));
    }
}

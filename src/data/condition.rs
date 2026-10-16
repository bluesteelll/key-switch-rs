//! Foreground-context predicates used to gate bindings.
//!
//! A `Condition` decides whether a binding fires given the currently focused
//! window's executable name and title. It reads them through the
//! [`ForegroundInfo`] trait, so the decision core in `crate::engine` can
//! evaluate conditions against a fake foreground in tests. The hook adapter
//! backs the trait with a [`ForegroundContext`] captured at most once per
//! keystroke, and only when a binding has a condition other than `Always`
//! (`Condition::Always` never touches the context).
//!
//! Every string comparison ignores case with Unicode lowercasing
//! (`str::to_lowercase`), so Cyrillic or Greek titles match in any case.
//! Windows treats exe and path names case-insensitively, and humans don't
//! reliably remember the casing of window titles either.

use windows::{
    Win32::{
        Foundation::*,
        System::Threading::{
            OpenProcess, PROCESS_NAME_NATIVE, PROCESS_QUERY_LIMITED_INFORMATION,
            QueryFullProcessImageNameW,
        },
        UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId},
    },
    core::{Owned, PWSTR},
};

/// Equality is structural, the condition as written: `AppEquals("Code.exe")`
/// and `AppEquals("code.exe")` differ. The engine uses it to group Tap, Hold
/// and DoubleTap bindings on the same keys, and the loader to find duplicate
/// bindings (roadmap item 1.6).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Condition {
    /// Always true — the binding fires whenever the combo matches. Default
    /// when the config doesn't specify `when:`.
    #[default]
    Always,
    /// True when the file name of the focused window's executable, `.exe`
    /// included, equals the string (case-insensitive, Unicode-aware), so
    /// `"code.exe"` matches `C:\...\Microsoft VS Code\Code.exe`.
    AppEquals(String),
    /// True when the focused window's title contains the substring
    /// (case-insensitive, Unicode-aware). Useful for picking out an app by
    /// tab name even when many windows share the same executable.
    TitleContains(String),
    /// True when the focused window's title is exactly the given string
    /// (case-insensitive, Unicode-aware).
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

    pub fn evaluate(&self, ctx: &dyn ForegroundInfo) -> bool {
        match self {
            Condition::Always => true,
            Condition::AppEquals(target) => match ctx.app() {
                Some(actual) => equal_ignoring_case(actual, target),
                None => false,
            },
            Condition::TitleContains(needle) => match ctx.title() {
                Some(actual) => actual.to_lowercase().contains(&needle.to_lowercase()),
                None => false,
            },
            Condition::TitleEquals(target) => match ctx.title() {
                Some(actual) => equal_ignoring_case(actual, target),
                None => false,
            },
            Condition::Not(inner) => !inner.evaluate(ctx),
            Condition::And(parts) => parts.iter().all(|c| c.evaluate(ctx)),
            Condition::Or(parts) => parts.iter().any(|c| c.evaluate(ctx)),
        }
    }
}

/// Equality under the lowercasing `TitleContains` uses, so a title that
/// `TitleEquals(s)` accepts also contains `s`. ASCII strings lowercase byte
/// by byte, so two of them compare without allocating.
fn equal_ignoring_case(a: &str, b: &str) -> bool {
    if a.is_ascii() && b.is_ascii() {
        a.eq_ignore_ascii_case(b)
    } else {
        a.to_lowercase() == b.to_lowercase()
    }
}

/// What a [`Condition`] can ask about the focused window. Both answers are
/// `None` when there is no foreground window or it cannot be inspected.
pub trait ForegroundInfo {
    /// Base name of the focused window's executable, e.g. `"chrome.exe"`.
    fn app(&self) -> Option<&str>;
    /// Title bar text of the focused window.
    fn title(&self) -> Option<&str>;
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
        self.app.get_or_init(|| self.fetch_app()).as_deref()
    }

    /// Title bar text of the focused window, or `None` if the window is
    /// gone or has no title (rare).
    pub fn title(&self) -> Option<&str> {
        self.title.get_or_init(|| self.fetch_title()).as_deref()
    }

    /// `None` when there is no window, its process is gone or cannot be
    /// opened, or its image path has 1024 or more UTF-16 units. Unpaired
    /// surrogates in the file name come back as U+FFFD.
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
        process_image_name(pid).ok()
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

impl ForegroundInfo for ForegroundContext {
    fn app(&self) -> Option<&str> {
        ForegroundContext::app(self)
    }

    fn title(&self) -> Option<&str> {
        ForegroundContext::title(self)
    }
}

/// Capacity, in UTF-16 units, of the buffer for a process image path: four
/// times MAX_PATH, so long-path installs fit without a heap retry.
const IMAGE_PATH_CAPACITY: usize = 1024;

/// File name of the executable of process `pid`, e.g. `"notepad.exe"`.
fn process_image_name(pid: u32) -> windows::core::Result<String> {
    // QueryFullProcessImageNameW needs only PROCESS_QUERY_LIMITED_INFORMATION,
    // which Windows also grants against elevated and protected processes.
    // Reading the module list needs PROCESS_VM_READ on top, so it failed on
    // this handle for every process (KS-03).
    //
    // SAFETY: OpenProcess takes no pointers. In `Ok` it returns a valid
    // handle that nothing else owns or closes, so `Owned` may close it.
    let process =
        unsafe { Owned::new(OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)?) };

    let mut buffer = [0u16; IMAGE_PATH_CAPACITY];
    let mut size = IMAGE_PATH_CAPACITY as u32;
    // The native form (`\Device\HarddiskVolume3\...`) ends in the same file
    // name as the Win32 form and skips its device-to-drive-letter
    // translation: about 2 us per lookup instead of 20 us, on the hook
    // thread.
    //
    // SAFETY: `*process` is a live handle with the access right the call
    // needs. The PWSTR points at `buffer`, which outlives the call, and
    // `size` holds its capacity in UTF-16 units, so the call writes within
    // it.
    unsafe {
        QueryFullProcessImageNameW(
            *process,
            PROCESS_NAME_NATIVE,
            PWSTR(buffer.as_mut_ptr()),
            &mut size,
        )?;
    }
    let path = &buffer[..(size as usize).min(IMAGE_PATH_CAPACITY)];
    let file_name =
        image_file_name(path).ok_or_else(|| windows::core::Error::from(ERROR_INVALID_DATA))?;
    Ok(String::from_utf16_lossy(file_name))
}

/// The UTF-16 units of `path` after its last `\`, or `None` when there are
/// none.
fn image_file_name(path: &[u16]) -> Option<&[u16]> {
    path.rsplit(|&unit| unit == u16::from(b'\\'))
        .next()
        .filter(|name| !name.is_empty())
}

#[cfg(test)]
mod tests {
    use windows::{
        Win32::UI::WindowsAndMessaging::{
            CreateWindowExW, DestroyWindow, HWND_MESSAGE, WINDOW_EX_STYLE, WINDOW_STYLE,
        },
        core::{PCWSTR, w},
    };

    use super::*;

    /// A foreground that needs no window: what the engine's tests use.
    struct FakeForeground {
        app: Option<&'static str>,
        title: Option<&'static str>,
    }

    impl ForegroundInfo for FakeForeground {
        fn app(&self) -> Option<&str> {
            self.app
        }

        fn title(&self) -> Option<&str> {
            self.title
        }
    }

    #[test]
    fn leaf_conditions_read_the_foreground_info() {
        let code = FakeForeground {
            app: Some("Code.exe"),
            title: Some("main.rs - Visual Studio Code"),
        };
        assert!(Condition::AppEquals("code.exe".into()).evaluate(&code));
        assert!(!Condition::AppEquals("chrome.exe".into()).evaluate(&code));
        assert!(Condition::TitleContains("visual studio".into()).evaluate(&code));
        assert!(Condition::TitleEquals("MAIN.RS - visual studio code".into()).evaluate(&code));
        assert!(!Condition::TitleEquals("main.rs".into()).evaluate(&code));

        // No foreground window: every leaf is false, so `Not` is true.
        let nothing = FakeForeground {
            app: None,
            title: None,
        };
        assert!(!Condition::AppEquals("code.exe".into()).evaluate(&nothing));
        assert!(!Condition::TitleContains("".into()).evaluate(&nothing));
        assert!(
            Condition::Not(Box::new(Condition::AppEquals("code.exe".into()))).evaluate(&nothing)
        );
    }

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
            Condition::Always,                           // not evaluated, but must not panic
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

    /// File name of this test binary, e.g. `key_switch_rs-<hash>.exe`.
    fn test_exe_file_name() -> String {
        std::env::current_exe()
            .expect("invariant: the test binary knows its own path")
            .file_name()
            .expect("invariant: an executable path ends in a file name")
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn process_image_name_resolves_the_current_process() {
        let name = process_image_name(std::process::id())
            .unwrap_or_else(|e| panic!("process_image_name(own pid) failed: {e}"));
        assert!(
            name.eq_ignore_ascii_case(&test_exe_file_name()),
            "{name:?} is not the test binary's file name"
        );
        assert!(!name.contains('\\'), "{name:?} is a path, not a file name");
        assert!(
            name.to_ascii_lowercase().ends_with(".exe"),
            "{name:?} does not end in .exe"
        );
    }

    #[test]
    fn app_equals_matches_the_current_process() {
        struct OwnProcess(Option<String>);

        impl ForegroundInfo for OwnProcess {
            fn app(&self) -> Option<&str> {
                self.0.as_deref()
            }

            fn title(&self) -> Option<&str> {
                None
            }
        }

        let own = OwnProcess(process_image_name(std::process::id()).ok());
        let exe = test_exe_file_name();
        assert!(
            Condition::AppEquals(exe.to_ascii_uppercase()).evaluate(&own),
            "AppEquals matches the test's own process (KS-03)"
        );
        assert!(
            Condition::And(vec![Condition::AppEquals(exe.clone()), Condition::Always])
                .evaluate(&own)
        );
        assert!(!Condition::Not(Box::new(Condition::AppEquals(exe))).evaluate(&own));
        assert!(!Condition::AppEquals("notepad.exe".into()).evaluate(&own));
    }

    /// A message-only window of the test thread: never shown or activated,
    /// and it gets no broadcasts. It gives `ForegroundContext` a real window
    /// of this process without touching the foreground.
    struct MessageOnlyWindow(HWND);

    impl MessageOnlyWindow {
        const TITLE: &str = "key-switch-rs condition test";

        fn create() -> Self {
            let title: Vec<u16> = Self::TITLE
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            // SAFETY: STATIC is a system class that user32 registers in every
            // process; `title` is a NUL-terminated UTF-16 buffer that lives
            // until the call returns; HWND_MESSAGE as the parent makes the
            // window message-only, so it is never shown or activated.
            let hwnd = unsafe {
                CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    w!("STATIC"),
                    PCWSTR(title.as_ptr()),
                    WINDOW_STYLE::default(),
                    0,
                    0,
                    0,
                    0,
                    Some(HWND_MESSAGE),
                    None,
                    None,
                    None,
                )
            }
            .unwrap_or_else(|e| panic!("CreateWindowExW(HWND_MESSAGE) failed: {e}"));
            Self(hwnd)
        }
    }

    impl Drop for MessageOnlyWindow {
        fn drop(&mut self) {
            // SAFETY: self.0 was created in `create` and nothing else destroys
            // it. HWND wraps a raw pointer, so this type is not Send and the
            // drop runs on the creating thread, as DestroyWindow requires.
            let _ = unsafe { DestroyWindow(self.0) };
        }
    }

    /// The type the hook adapter uses resolves a real window, so a
    /// `fetch_app` that loses the process name fails here, not only in a
    /// manual check.
    #[test]
    fn foreground_context_reads_a_window_of_this_process() {
        let window = MessageOnlyWindow::create();
        let ctx = ForegroundContext {
            hwnd: window.0,
            app: std::cell::OnceCell::new(),
            title: std::cell::OnceCell::new(),
        };
        let exe = test_exe_file_name();
        assert!(
            ctx.app().is_some_and(|app| app.eq_ignore_ascii_case(&exe)),
            "ForegroundContext::app() of a window of this process is {:?}, not {exe:?} (KS-03)",
            ctx.app()
        );
        assert!(Condition::AppEquals(exe).evaluate(&ctx));
        assert_eq!(ctx.title(), Some(MessageOnlyWindow::TITLE));
    }

    #[test]
    fn equals_conditions_fold_case_like_title_contains() {
        // "Kalkulyator" and "Privet.exe", written in Cyrillic.
        let title = "\u{41a}\u{430}\u{43b}\u{44c}\u{43a}\u{443}\u{43b}\u{44f}\u{442}\u{43e}\u{440}";
        let app = "\u{41f}\u{440}\u{438}\u{432}\u{435}\u{442}.exe";
        let cyrillic = FakeForeground {
            app: Some(app),
            title: Some(title),
        };
        assert_ne!(title.to_uppercase(), title);
        assert!(Condition::TitleContains(title.to_uppercase()).evaluate(&cyrillic));
        assert!(
            Condition::TitleEquals(title.to_uppercase()).evaluate(&cyrillic),
            "TitleEquals ignores Cyrillic case like TitleContains"
        );
        assert!(
            Condition::AppEquals(app.to_uppercase()).evaluate(&cyrillic),
            "AppEquals ignores Cyrillic case"
        );
        assert!(!Condition::TitleEquals("\u{41a}\u{430}\u{43b}".into()).evaluate(&cyrillic));

        // Lowercasing turns a word-final capital sigma into the final form,
        // so a per-char fold would miss what TitleContains finds.
        let greek = FakeForeground {
            app: None,
            title: Some("\u{39f}\u{394}\u{39f}\u{3a3}"),
        };
        let final_sigma = "\u{3bf}\u{3b4}\u{3bf}\u{3c2}";
        assert!(Condition::TitleContains(final_sigma.into()).evaluate(&greek));
        assert!(Condition::TitleEquals(final_sigma.into()).evaluate(&greek));

        // The Kelvin sign lowercases to an ASCII k: one non-ASCII side is
        // enough to need the Unicode comparison.
        let kelvin = FakeForeground {
            app: Some("\u{212a}.exe"),
            title: None,
        };
        assert!(Condition::AppEquals("k.exe".into()).evaluate(&kelvin));
        assert!(Condition::AppEquals("K.EXE".into()).evaluate(&kelvin));
    }

    #[test]
    fn image_file_name_is_the_part_after_the_last_backslash() {
        let file_name = |path: &str| {
            let path: Vec<u16> = path.encode_utf16().collect();
            image_file_name(&path).map(String::from_utf16_lossy)
        };
        assert_eq!(
            file_name(r"C:\Windows\System32\notepad.exe").as_deref(),
            Some("notepad.exe")
        );
        assert_eq!(
            file_name(r"\Device\HarddiskVolume3\Apps\App.EXE").as_deref(),
            Some("App.EXE")
        );
        assert_eq!(file_name("notepad.exe").as_deref(), Some("notepad.exe"));
        assert_eq!(file_name(r"C:\dir\"), None);
        assert_eq!(file_name(""), None);
    }

    #[test]
    #[ignore = "manual: start it, then focus a Notepad window within 5 s; it expects ForegroundContext::capture().app() to be notepad.exe in any case"]
    fn foreground_context_names_the_focused_notepad() {
        std::thread::sleep(std::time::Duration::from_secs(5));
        let ctx = ForegroundContext::capture();
        assert_eq!(
            ctx.app().map(str::to_ascii_lowercase).as_deref(),
            Some("notepad.exe")
        );
    }
}

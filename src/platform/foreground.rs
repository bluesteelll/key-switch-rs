//! The foreground window as `when:` conditions see it (roadmap item 2.3,
//! the latency bullet moved from 3.5).
//!
//! [`ForegroundCache`] keeps the executable name of the foreground window,
//! read once per (window, process) when the window takes the foreground:
//! the daemon fills it from `EVENT_SYSTEM_FOREGROUND`, right after the
//! hooks are installed, at `swch on` and when keyboard input comes back to
//! this desktop, on the main thread, never in the keyboard callback. The
//! keyboard callback reads the name from the cache, with no Win32 call and
//! no allocation.
//!
//! The title is read live, because it changes inside one window (a browser
//! tab), at most once per key-down that evaluates a title condition.
//! `GetWindowTextW` on a window of another process sends no `WM_GETTEXT`:
//! "This behavior is by design. It allows applications to call
//! GetWindowText without becoming unresponsive if the process that owns
//! the target window is not responding" (Microsoft Learn,
//! GetWindowTextW). On a window of this process it would send one and run
//! this thread's window procedures inside the hook, so such a window is
//! never asked.
//!
//! [`WindowQuery`] is the seam: [`Win32Query`] asks Windows, and a test
//! counts the questions. Nothing here imports the rest of the crate.

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

/// What the cache asks Windows about windows and processes.
pub trait WindowQuery {
    /// The foreground window, or an invalid handle if there is none.
    fn foreground(&self) -> HWND;
    /// The id of the process that owns `window`, 0 for none.
    fn owner(&self, window: HWND) -> u32;
    /// The file name of the executable of process `pid`, or `None` if it
    /// cannot be read.
    fn image_name(&self, pid: u32) -> Option<String>;
    /// The title of `window`, or `None` if it has none.
    fn title(&self, window: HWND) -> Option<String>;
}

/// [`WindowQuery`] by the Win32 calls.
pub struct Win32Query;

impl WindowQuery for Win32Query {
    fn foreground(&self) -> HWND {
        // SAFETY: GetForegroundWindow takes no arguments and reads no memory
        // of ours; with no foreground window it returns a null handle.
        unsafe { GetForegroundWindow() }
    }

    fn owner(&self, window: HWND) -> u32 {
        let mut pid = 0;
        // SAFETY: `pid` is a local that outlives the call, which only writes
        // it; a handle that names no window leaves it 0.
        let _thread = unsafe { GetWindowThreadProcessId(window, Some(&mut pid)) };
        pid
    }

    fn image_name(&self, pid: u32) -> Option<String> {
        process_image_name(pid).ok()
    }

    fn title(&self, window: HWND) -> Option<String> {
        window_title(window)
    }
}

/// The executable name of the foreground window, cached per window. Not
/// `Send`: it holds a window handle and belongs to the main thread.
pub struct ForegroundCache {
    query: Box<dyn WindowQuery>,
    /// The window the cache is for; invalid until the first one.
    window: HWND,
    /// The process that owned `window` when it was read.
    pid: u32,
    /// Whether `window` belongs to this process.
    own: bool,
    app: Option<String>,
}

impl Default for ForegroundCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ForegroundCache {
    /// A cache that asks Windows, for no window yet.
    pub fn new() -> Self {
        Self {
            query: Box::new(Win32Query),
            window: HWND::default(),
            pid: 0,
            own: false,
            app: None,
        }
    }

    /// A cache that asks `query`, for no window yet.
    #[cfg(test)]
    pub(crate) fn with_query(query: Box<dyn WindowQuery>) -> Self {
        Self {
            query,
            ..Self::new()
        }
    }

    /// `window` took the foreground. A handle that names no window, or no
    /// window any process owns now, keeps the cache, as during a desktop
    /// switch. The same window of the same process asks nothing; a handle
    /// that another process owns now is another window. A window of this
    /// process has no executable name for `when:`.
    pub fn window_changed(&mut self, window: HWND) {
        if window.is_invalid() {
            return;
        }
        let pid = self.query.owner(window);
        if pid == 0 || (window == self.window && pid == self.pid) {
            return;
        }
        self.window = window;
        self.pid = pid;
        self.own = pid == std::process::id();
        self.app = if self.own {
            None
        } else {
            self.query.image_name(pid)
        };
    }

    /// Reads the foreground window again, for when no foreground event
    /// may have been seen: right after the hooks are installed, at `swch
    /// on` and when keyboard input comes back to this desktop.
    pub fn resync(&mut self) {
        let window = self.query.foreground();
        self.window_changed(window);
    }

    /// The executable name of the foreground window (`"code.exe"`), with no
    /// Win32 call.
    pub fn app(&self) -> Option<&str> {
        self.app.as_deref()
    }

    /// The title of the foreground window, read now. `None` with no
    /// window, and for a window of this process, which is never asked.
    pub fn read_title(&self) -> Option<String> {
        if self.window.is_invalid() || self.own {
            return None;
        }
        self.query.title(self.window)
    }
}

/// Capacity, in UTF-16 units, of the buffer for a window title. A longer
/// title is cut there.
const TITLE_CAPACITY: usize = 512;

/// The title of `window`, or `None` if it has none or is gone.
fn window_title(window: HWND) -> Option<String> {
    let mut buffer = [0u16; TITLE_CAPACITY];
    // SAFETY: `buffer` is a local that outlives the call, and the wrapper
    // passes its length, so the call writes within it; a handle that names
    // no window returns 0.
    let len = unsafe { GetWindowTextW(window, &mut buffer) };
    let len = usize::try_from(len).ok().filter(|&len| len > 0)?;
    Some(String::from_utf16_lossy(&buffer[..len.min(TITLE_CAPACITY)]))
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
    // translation: about 2 us per lookup instead of 20 us.
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
    use std::cell::Cell;
    use std::rc::Rc;

    use windows::{
        Win32::{
            Foundation::HWND,
            UI::WindowsAndMessaging::{
                CreateWindowExW, DestroyWindow, HWND_MESSAGE, WINDOW_EX_STYLE, WINDOW_STYLE,
            },
        },
        core::{PCWSTR, w},
    };

    use super::*;
    use crate::data::condition::{Condition, ForegroundInfo};

    /// A window handle for the fake queries. No Win32 call ever sees it.
    fn window(raw: usize) -> HWND {
        HWND(raw as *mut std::ffi::c_void)
    }

    /// Windows 1 and 2 of process 10 and window 3 of process 20, with
    /// counted process queries; window 4 has no owner.
    struct FakeQuery {
        recycled: Rc<Cell<bool>>,
        image_queries: Rc<Cell<u32>>,
    }

    impl WindowQuery for FakeQuery {
        fn foreground(&self) -> HWND {
            window(3)
        }
        fn owner(&self, window: HWND) -> u32 {
            match window.0 as usize {
                1 if self.recycled.get() => 30,
                1 | 2 => 10,
                3 => 20,
                _ => 0,
            }
        }
        fn image_name(&self, pid: u32) -> Option<String> {
            self.image_queries.set(self.image_queries.get() + 1);
            Some(format!("p{pid}.exe"))
        }
        fn title(&self, window: HWND) -> Option<String> {
            Some(format!("window {}", window.0 as usize))
        }
    }

    fn fake_cache() -> (ForegroundCache, Rc<Cell<bool>>, Rc<Cell<u32>>) {
        let recycled = Rc::new(Cell::new(false));
        let image_queries = Rc::new(Cell::new(0));
        let cache = ForegroundCache::with_query(Box::new(FakeQuery {
            recycled: Rc::clone(&recycled),
            image_queries: Rc::clone(&image_queries),
        }));
        (cache, recycled, image_queries)
    }

    #[test]
    fn the_cache_queries_again_only_for_another_window() {
        // Roadmap item 2.3 (latency): the executable name is read once per
        // window, when the window takes the foreground, and never on the key
        // path.
        let (mut cache, recycled, image_queries) = fake_cache();
        assert_eq!(cache.app(), None, "no window yet");
        assert_eq!(cache.read_title(), None);

        cache.window_changed(window(1));
        assert_eq!(cache.app(), Some("p10.exe"));
        cache.window_changed(window(1));
        assert_eq!(image_queries.get(), 1, "the same window asks nothing");
        cache.window_changed(window(2));
        assert_eq!(image_queries.get(), 2, "another window of the same process");
        cache.window_changed(window(3));
        assert_eq!(cache.app(), Some("p20.exe"));
        assert_eq!(cache.read_title().as_deref(), Some("window 3"));

        cache.window_changed(window(1));
        recycled.set(true);
        cache.window_changed(window(1));
        assert_eq!(
            cache.app(),
            Some("p30.exe"),
            "a handle that another process owns now is another window"
        );
        assert_eq!(image_queries.get(), 5);

        cache.resync();
        assert_eq!(cache.app(), Some("p20.exe"), "resync reads the foreground");
    }

    #[test]
    fn an_invalid_foreground_window_keeps_the_cache() {
        // EVENT_SYSTEM_FOREGROUND can name no window, or one that is gone,
        // during a desktop switch: the last window stays.
        let (mut cache, _, image_queries) = fake_cache();
        cache.window_changed(window(1));
        cache.window_changed(HWND::default());
        cache.window_changed(window(4));
        assert_eq!(cache.app(), Some("p10.exe"));
        assert_eq!(cache.read_title().as_deref(), Some("window 1"));
        assert_eq!(image_queries.get(), 1);
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

        let own = OwnProcess(Win32Query.image_name(std::process::id()));
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
    /// and it gets no broadcasts. It gives the queries a real window of
    /// this process without touching the foreground.
    struct MessageOnlyWindow(HWND);

    impl MessageOnlyWindow {
        const TITLE: &str = "key-switch-rs foreground test";

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

    /// The Win32 queries resolve a real window, so a query that loses the
    /// process name fails here, not only in a manual check.
    #[test]
    fn the_win32_queries_read_a_window_of_this_process() {
        let window = MessageOnlyWindow::create();
        let pid = Win32Query.owner(window.0);
        assert_eq!(pid, std::process::id());
        let exe = test_exe_file_name();
        assert!(
            Win32Query
                .image_name(pid)
                .is_some_and(|app| app.eq_ignore_ascii_case(&exe)),
            "the image name of this process is not {exe:?} (KS-03)"
        );
        // A window of this thread: GetWindowTextW calls its procedure
        // directly, outside any hook.
        assert_eq!(
            Win32Query.title(window.0).as_deref(),
            Some(MessageOnlyWindow::TITLE)
        );
    }

    #[test]
    fn a_window_of_this_process_has_no_app_and_is_not_asked_for_its_title() {
        // Reading the title of a window of this process would send it
        // WM_GETTEXT, which the keyboard hook must never do (it would run
        // this thread's window procedures under the main state's borrow).
        let window = MessageOnlyWindow::create();
        let mut cache = ForegroundCache::new();
        cache.window_changed(window.0);
        assert_eq!(cache.app(), None);
        assert_eq!(cache.read_title(), None);
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
    #[ignore = "manual: start it, then focus a Notepad window within 5 s; it expects the foreground cache to name notepad.exe in any case"]
    fn the_foreground_cache_names_the_focused_notepad() {
        std::thread::sleep(std::time::Duration::from_secs(5));
        let mut cache = ForegroundCache::new();
        cache.resync();
        assert_eq!(
            cache.app().map(str::to_ascii_lowercase).as_deref(),
            Some("notepad.exe")
        );
    }
}

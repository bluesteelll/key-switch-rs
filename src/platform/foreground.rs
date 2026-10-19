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

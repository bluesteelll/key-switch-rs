#[cfg(test)]
mod tests {
    use std::sync::PoisonError;

    use windows::Win32::{Foundation::*, UI::WindowsAndMessaging::*};
    use windows::core::BOOL;

    use super::*;
    use crate::engine::InputSignal;
    use crate::hook::keyboard_hook_callback::{GLOBAL_HOOK, get_hook};
    use crate::hook::main_thread::{MainThread, take_pending_signals, with_main};

    // No test here installs a hook, changes a session or injects input. The
    // window procedure is called directly, with no window, and the one test
    // that creates the control window destroys it again at once.

    /// Turns the process-wide hook's remapping back on when dropped, also on
    /// unwind, so a failed test leaves the next one remapping on.
    struct RemappingOnAtEnd;

    impl Drop for RemappingOnAtEnd {
        fn drop(&mut self) {
            let _ = with_main(|main| main.set_enabled(get_hook(), true));
        }
    }

    /// Calls the window procedure with `msg`, as a dispatch would.
    fn call(msg: u32, wparam: usize) -> LRESULT {
        // SAFETY: the procedure reads only `msg` and `wparam` for the
        // messages these tests send, and passes nothing to a window: the
        // null handle is never used for them.
        unsafe { control_window_proc(HWND::default(), msg, WPARAM(wparam), LPARAM(0)) }
    }

    /// The signals noted since the last call, in `InputSignal::ALL` order.
    fn noted() -> Vec<InputSignal> {
        take_pending_signals()
            .map(|pending| pending.signals().collect())
            .unwrap_or_default()
    }

    #[test]
    fn only_the_lock_and_the_unlock_are_input_signals() {
        assert_eq!(
            session_signal(WPARAM(WTS_SESSION_LOCK as usize)),
            Some(InputSignal::SessionLocked)
        );
        assert_eq!(
            session_signal(WPARAM(WTS_SESSION_UNLOCK as usize)),
            Some(InputSignal::SessionUnlocked)
        );
        for other in [
            WTS_SESSION_LOGON,
            WTS_CONSOLE_CONNECT,
            WTS_REMOTE_DISCONNECT,
        ] {
            assert_eq!(session_signal(WPARAM(other as usize)), None, "{other}");
        }
    }

    #[test]
    fn the_session_messages_note_the_lock_and_the_unlock() {
        // Critique M6: the procedure's session wiring, with remapping on and
        // off. The procedure reads the process-wide hook's switch.
        let _global = GLOBAL_HOOK.lock().unwrap_or_else(PoisonError::into_inner);
        let _main = MainThread::new().expect("the main thread is set up");
        let _on_at_end = RemappingOnAtEnd;

        assert_eq!(
            call(WM_WTSSESSION_CHANGE, WTS_SESSION_LOCK as usize),
            LRESULT(0)
        );
        assert_eq!(noted(), [InputSignal::SessionLocked]);
        call(WM_WTSSESSION_CHANGE, WTS_SESSION_UNLOCK as usize);
        assert_eq!(noted(), [InputSignal::SessionUnlocked]);
        call(WM_WTSSESSION_CHANGE, WTS_SESSION_LOGON as usize);
        assert_eq!(noted(), []);

        assert!(
            with_main(|main| main.set_enabled(get_hook(), false)).expect("the main state is free")
        );
        call(WM_WTSSESSION_CHANGE, WTS_SESSION_LOCK as usize);
        assert_eq!(noted(), [], "with remapping off nothing is noted");
    }

    #[test]
    fn the_session_end_messages_are_answered_and_end_the_loop() {
        // Critique M6 and ruling Q5: the procedure answers
        // WM_QUERYENDSESSION with TRUE, and at WM_ENDSESSION(TRUE) removes
        // the hook (none is installed here, so that is a no-op), logs one
        // line, flushes (no writer runs in tests, so at once) and posts
        // WM_QUIT, which the test takes off its own queue again.
        let _global = GLOBAL_HOOK.lock().unwrap_or_else(PoisonError::into_inner);
        let _main = MainThread::new().expect("the main thread is set up");
        let quit_queued = || {
            let mut msg = MSG::default();
            // SAFETY: `msg` is a local that outlives the call, which only
            // writes it and removes at most the WM_QUIT of this thread.
            unsafe { PeekMessageW(&mut msg, None, WM_QUIT, WM_QUIT, PM_REMOVE) }.as_bool()
        };

        assert_eq!(call(WM_QUERYENDSESSION, 0), LRESULT(1));
        assert_eq!(call(WM_ENDSESSION, 0), LRESULT(0));
        assert!(!quit_queued(), "a session that does not end ends nothing");

        let (answer, lines) = crate::logging::testing::capture(|| call(WM_ENDSESSION, 1));
        assert_eq!(answer, LRESULT(0));
        assert!(quit_queued(), "the loop is asked to end");
        assert!(
            lines.iter().any(|line| line
                .contains("the session is ending; removing the keyboard hook and exiting")),
            "{lines:?}"
        );
        assert!(!get_hook().is_installed());
    }

    /// Counts the windows of the calling thread.
    unsafe extern "system" fn count_window(_window: HWND, count: LPARAM) -> BOOL {
        // SAFETY: `count` is the address of the `usize` that
        // `windows_of_this_thread` passes, alive for the whole enumeration.
        unsafe { *(count.0 as *mut usize) += 1 };
        BOOL(1)
    }

    /// How many windows the calling thread has (`EnumThreadWindows`, which
    /// only reads).
    fn windows_of_this_thread() -> usize {
        let mut count = 0usize;
        // SAFETY: the callback only adds to `count`, a local that outlives
        // the call, through the address passed as its LPARAM.
        let _ = unsafe {
            EnumThreadWindows(
                windows::Win32::System::Threading::GetCurrentThreadId(),
                Some(count_window),
                LPARAM(&mut count as *mut usize as isize),
            )
        };
        count
    }

    #[test]
    fn the_control_window_is_a_hidden_top_level_window_released_on_drop() {
        // Critique M5: the session registration may be unavailable where
        // Remote Desktop Services have not started; both outcomes are named.
        // Critic OQ1: once the window is dropped the thread has no window
        // left, the default IME window included. The class is registered
        // once per process, so this test and bench row R13 take one lock.
        let _class = CONTROL_CLASS.lock().unwrap_or_else(PoisonError::into_inner);
        assert_eq!(windows_of_this_thread(), 0, "a fresh test thread");

        let (window, lines) = crate::logging::testing::capture(ControlWindow::create);
        let window = window.expect("the control window is created");
        let hwnd = window.hwnd();
        // SAFETY: the three calls take the handle by value and only read
        // the window, which lives until `window` is dropped below.
        let (exists, visible, parent, desktop) = unsafe {
            (
                IsWindow(Some(hwnd)).as_bool(),
                IsWindowVisible(hwnd).as_bool(),
                GetAncestor(hwnd, GA_PARENT),
                GetDesktopWindow(),
            )
        };
        assert!(exists);
        assert!(!visible, "the window is never shown");
        assert_eq!(
            parent, desktop,
            "a top-level window, not a message-only one"
        );
        match window.session_notifications() {
            SessionNotifications::Registered => {}
            SessionNotifications::Unavailable => assert!(
                lines
                    .iter()
                    .any(|line| line.contains("could not register for session notifications")),
                "{lines:?}"
            ),
        }

        drop(window);
        // SAFETY: IsWindow only reads the handle's value; a destroyed window
        // reads as no window.
        assert!(
            !unsafe { IsWindow(Some(hwnd)) }.as_bool(),
            "destroyed on drop"
        );
        assert_eq!(
            windows_of_this_thread(),
            0,
            "no window of this thread outlives the control window"
        );
        let again = ControlWindow::create().expect("the class was unregistered on drop");
        drop(again);
    }
}

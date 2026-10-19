//! The daemon's control window (roadmap item 2.3): a hidden top-level
//! window owned by the main thread, which receives what Windows sends to
//! windows and not to threads.
//!
//! - `WM_WTSSESSION_CHANGE`, once the window is registered for the
//!   notifications of this session (`WTSRegisterSessionNotification`): a
//!   lock or an unlock is noted for the main loop as an input signal
//!   (`main_thread::note_input_signal`). The loop resets the transient state
//!   when keyboard input left this desktop; the procedure itself resets
//!   nothing, because it may run inside another message call.
//! - `WM_QUERYENDSESSION` is answered with TRUE, and at `WM_ENDSESSION` for
//!   a session that ends the daemon removes its hooks, logs one line,
//!   flushes the log and ends its loop.
//!
//! It is not a message-only window (`HWND_MESSAGE`), which "does not
//! receive broadcast messages": the tray of roadmap item 4.6 needs the
//! `TaskbarCreated` broadcast. It is never shown or activated and has no
//! taskbar or Alt+Tab entry. `MAIN_WAKE` stays a thread message.
//!
//! Every handle has a guard, dropped in the order Windows asks for: the
//! session registration first ("it must call
//! WTSUnRegisterSessionNotification before being destroyed"), then the
//! window, on the thread that made it, then the class, which
//! `UnregisterClassW` refuses while a window of it exists. The window must
//! not outlive the loop that pumps it: a `SendMessage` to a top-level window
//! whose thread no longer pumps would hang its sender, so `App::run` drops
//! it right after the hooks are removed, before the executor's bounded
//! stop.

use windows::{
    Win32::{
        Foundation::*,
        System::{
            LibraryLoader::GetModuleHandleW,
            RemoteDesktop::{
                NOTIFY_FOR_THIS_SESSION, WTSRegisterSessionNotification,
                WTSUnRegisterSessionNotification,
            },
        },
        UI::WindowsAndMessaging::*,
    },
    core::{Error, PCWSTR, w},
};

use crate::engine::InputSignal;
use crate::hook::keyboard_hook::KeyboardHook;
use crate::hook::keyboard_hook_callback::get_hook;
use crate::hook::main_thread::note_input_signal;
use crate::logging;

/// The class of the control window, registered for this process.
const CLASS_NAME: PCWSTR = w!("key-switch-rs control");

/// Taken by every test that registers the control window's class, which
/// exists once per process: the window test and bench row R13.
#[cfg(test)]
pub(crate) static CONTROL_CLASS: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Whether the control window gets the session notifications. The daemon
/// only needs the warning `create` logs; the tests name both outcomes.
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SessionNotifications {
    Registered,
    /// `WTSRegisterSessionNotification` failed, for example before Remote
    /// Desktop Services have started: a lock is then answered only by the
    /// switch to the lock screen's desktop.
    Unavailable,
}

/// The control window and its registrations. Not `Send`: it holds a window
/// handle, and the window belongs to the thread that made it. The fields
/// drop in their order.
pub(crate) struct ControlWindow {
    _session: Option<SessionRegistration>,
    _window: OwnedWindow,
    _class: RegisteredClass,
}

/// The registered class; unregistered when dropped.
struct RegisteredClass {
    instance: HINSTANCE,
}

impl Drop for RegisteredClass {
    fn drop(&mut self) {
        // SAFETY: the class name is a static wide literal and `instance` is
        // the module handle the class was registered with; no window of the
        // class exists any more, because the window guard drops first.
        if let Err(error) = unsafe { UnregisterClassW(CLASS_NAME, Some(self.instance)) } {
            log::warn!("could not unregister the control window class: {error}");
        }
    }
}

/// The window; destroyed when dropped, on the thread that made it.
struct OwnedWindow(HWND);

impl Drop for OwnedWindow {
    fn drop(&mut self) {
        // SAFETY: `self.0` is the window `create` made on this thread, which
        // nothing else destroys; HWND is not Send, so this drop runs on the
        // creating thread, as DestroyWindow requires.
        if let Err(error) = unsafe { DestroyWindow(self.0) } {
            log::warn!("could not destroy the control window: {error}");
        }
    }
}

/// The window's registration for the session notifications; undone when
/// dropped, before the window is destroyed.
struct SessionRegistration(HWND);

impl Drop for SessionRegistration {
    fn drop(&mut self) {
        // SAFETY: `self.0` is the live window this registration was made
        // for; the window guard drops after this one.
        if let Err(error) = unsafe { WTSUnRegisterSessionNotification(self.0) } {
            log::warn!("could not unregister the session notifications: {error}");
        }
    }
}

impl ControlWindow {
    /// Registers the class, creates the window and registers it for the
    /// notifications of this session. An error if the class or the window
    /// cannot be made; a session registration that fails is logged and
    /// leaves [`SessionNotifications::Unavailable`].
    pub(crate) fn create() -> windows::core::Result<Self> {
        // SAFETY: a null module name asks for the handle of the file that
        // made this process; the call reads no memory of ours.
        let instance = HINSTANCE::from(unsafe { GetModuleHandleW(None) }?);
        let class = WNDCLASSW {
            lpfnWndProc: Some(control_window_proc),
            hInstance: instance,
            lpszClassName: CLASS_NAME,
            ..Default::default()
        };
        // SAFETY: `class` is a local that outlives the call, which only
        // reads it; its procedure is a 'static extern "system" fn and its
        // name a static wide literal.
        if unsafe { RegisterClassW(&class) } == 0 {
            return Err(Error::from_thread());
        }
        let class = RegisteredClass { instance };
        // SAFETY: the class was registered above with this module handle;
        // the window name is a static wide literal and no creation data is
        // passed. WS_POPUP without WS_VISIBLE is never shown,
        // WS_EX_TOOLWINDOW keeps it off the taskbar and Alt+Tab, and
        // WS_EX_NOACTIVATE keeps it from being activated.
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                CLASS_NAME,
                w!("key-switch-rs"),
                WS_POPUP,
                0,
                0,
                0,
                0,
                None,
                None,
                Some(instance),
                None,
            )
        }?;
        let window = OwnedWindow(hwnd);
        // SAFETY: `hwnd` is the live window made above, on this thread.
        let session = match unsafe { WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION) }
        {
            Ok(()) => Some(SessionRegistration(hwnd)),
            Err(error) => {
                log::warn!(
                    "could not register for session notifications ({error}); a lock is answered only by the switch to the lock screen's desktop"
                );
                None
            }
        };
        Ok(Self {
            _session: session,
            _window: window,
            _class: class,
        })
    }

    /// Whether the window gets the session notifications.
    #[cfg(test)]
    pub(crate) fn session_notifications(&self) -> SessionNotifications {
        match self._session {
            Some(_) => SessionNotifications::Registered,
            None => SessionNotifications::Unavailable,
        }
    }

    #[cfg(test)]
    pub(crate) fn hwnd(&self) -> HWND {
        self._window.0
    }
}

/// The input signal of a `WM_WTSSESSION_CHANGE` with `wparam`: the lock and
/// the unlock; every other session change is none.
fn session_signal(wparam: WPARAM) -> Option<InputSignal> {
    match u32::try_from(wparam.0).ok()? {
        WTS_SESSION_LOCK => Some(InputSignal::SessionLocked),
        WTS_SESSION_UNLOCK => Some(InputSignal::SessionUnlocked),
        _ => None,
    }
}

/// Whether a `WM_ENDSESSION` says that the session ends.
fn session_ends(wparam: WPARAM) -> bool {
    wparam.0 != 0
}

/// The session ends: "the session can end any time after all applications
/// have returned from processing this message" (Microsoft Learn,
/// WM_ENDSESSION), so the hooks go and the log is written before the
/// procedure returns, and the main loop is asked to end. On the main
/// thread, outside the main state's borrow and with the hooks removed
/// before the flush, which waits at most 1 s.
fn end_session(hook: &KeyboardHook) {
    if let Err(error) = hook.uninstall() {
        log::warn!("could not remove the keyboard hook: {error}");
    }
    log::info!("the session is ending; removing the keyboard hook and exiting");
    logging::flush();
    // SAFETY: PostQuitMessage takes the exit code by value and only marks
    // this thread's queue; the main loop ends at the WM_QUIT.
    unsafe { PostQuitMessage(0) };
}

/// The control window's procedure. It notes signals and answers the
/// session end; nothing in it can panic.
unsafe extern "system" fn control_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_WTSSESSION_CHANGE => {
            if let Some(signal) = session_signal(wparam)
                && get_hook().is_enabled()
            {
                note_input_signal(signal);
            }
            LRESULT(0)
        }
        // "Applications should respect the user's intentions and return
        // TRUE" (WM_QUERYENDSESSION).
        WM_QUERYENDSESSION => LRESULT(1),
        WM_ENDSESSION => {
            if session_ends(wparam) {
                end_session(get_hook());
            }
            LRESULT(0)
        }
        // Only the guard destroys the window.
        WM_CLOSE => LRESULT(0),
        // SAFETY: passes the message on with the exact arguments the system
        // gave this window procedure.
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

#[cfg(test)]
mod tests {
    use std::sync::PoisonError;

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

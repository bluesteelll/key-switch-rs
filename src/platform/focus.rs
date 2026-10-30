//! The dialog class and the focus window of a window's thread: what a
//! language request reads to reach the focused control of a dialog box
//! (owner report 2026-10-30).
//!
//! A dialog box such as Save As or Open ignores a `WM_INPUTLANGCHANGEREQUEST`
//! posted to the dialog itself, while the window that has the keyboard
//! focus in it, such as the File name box, accepts it. [`is_dialog`] tells
//! the system dialog class (`#32770`) by its class atom, and
//! [`focus_window_of`] finds the focus window of the thread that created a
//! window. "The window with the keyboard focus is either the active window,
//! or a child window of the active window" (Microsoft Learn, Keyboard Input
//! Overview), so for a dialog that is the foreground window it is the
//! dialog or one of its descendants.
//!
//! Every call here only reads, and none sends a message to the window it
//! asks about: GetGUIThreadInfo "is useful for retrieving out-of-context
//! information about a thread" and "succeeds even if the active window is
//! not owned by the calling process" (Microsoft Learn, GetGUIThreadInfo).
//! The focus it returns can be stale: it "may not return valid window
//! handles ... when called to retrieve information for the foreground
//! thread, such as when a window is losing activation", so a caller that
//! posts to the focus window must expect the post to fail. Nothing here
//! imports the rest of the crate.

use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
        GCW_ATOM, GUITHREADINFO, GetClassWord, GetGUIThreadInfo, GetWindowThreadProcessId,
    },
};

/// The class atom of the system dialog class, `#32770` ("The class for a
/// dialog box", Microsoft Learn, About Window Classes). An integer atom's
/// string form is `#dddd` with decimal digits (About Atom Tables), so this
/// is `WC_DIALOG`, `MAKEINTATOM(0x8002)` in WinUser.h, which the `windows`
/// crate does not define.
pub const DIALOG_CLASS_ATOM: u16 = 32770;

/// The atom of the class of `window` (`GetClassWord(GCW_ATOM)`), 0 if it
/// cannot be read, for example when `window` names no window.
pub fn class_atom(window: HWND) -> u16 {
    // SAFETY: GetClassWord takes the handle and the index by value and
    // writes no memory of ours; GCW_ATOM is a valid index for every window
    // class (the one index the function is not deprecated for), and a
    // handle that names no window makes it return 0.
    unsafe { GetClassWord(window, GCW_ATOM.0) }
}

/// Whether `window` is of the system dialog class (`#32770`). A window
/// whose class cannot be read is not.
pub fn is_dialog(window: HWND) -> bool {
    class_atom(window) == DIALOG_CLASS_ATOM
}

/// The window that has the keyboard focus in the thread that created
/// `window`, or `None` if that thread has none.
///
/// It asks the thread of `window`, never thread 0, which would be whatever
/// thread is in the foreground at the time of the read: a foreground change
/// after the caller read `window` cannot make it read another thread.
///
/// # Errors
///
/// When `window` names no window (its thread id is 0) or its thread has no
/// input queue. The error carries `GetLastError` (`Error::from_thread`),
/// which makes no COM call, so a caller on the hook path may build it.
pub fn focus_window_of(window: HWND) -> windows::core::Result<Option<HWND>> {
    // SAFETY: GetWindowThreadProcessId takes the handle by value, and with
    // no process id pointer it writes no memory of ours; a handle that names
    // no window makes it return 0.
    let thread = unsafe { GetWindowThreadProcessId(window, None) };
    if thread == 0 {
        return Err(windows::core::Error::from_thread());
    }
    let mut info = GUITHREADINFO {
        cbSize: size_of::<GUITHREADINFO>() as u32,
        ..Default::default()
    };
    // SAFETY: `info` is a local GUITHREADINFO that outlives the call, and its
    // cbSize is its own size, as the function requires, so the call writes
    // within it. The thread id is passed by value.
    unsafe { GetGUIThreadInfo(thread, &mut info) }?;
    Ok((!info.hwndFocus.is_invalid()).then_some(info.hwndFocus))
}

#[cfg(test)]
mod tests {
    use windows::{
        Win32::UI::WindowsAndMessaging::{
            CreateWindowExW, DestroyWindow, GetDesktopWindow, GetForegroundWindow, HWND_MESSAGE,
            WINDOW_EX_STYLE, WINDOW_STYLE,
        },
        core::{PCWSTR, w},
    };

    use super::*;

    /// The class atom of the desktop window's class, `#32769` ("The class
    /// for the desktop window", Microsoft Learn, About Window Classes).
    const DESKTOP_CLASS_ATOM: u16 = 32769;

    /// A message-only window of the test thread, of a system class: never
    /// shown or activated, and it gets no broadcasts. It gives the reads a
    /// real window of a known class without touching the foreground.
    struct MessageOnlyWindow(HWND);

    impl MessageOnlyWindow {
        fn create(class: PCWSTR) -> Self {
            // SAFETY: `class` is a compile-time literal (`w!`) that names a
            // system class user32 registers in every process (`#32770`,
            // `STATIC`); the window has no name; HWND_MESSAGE as the parent
            // makes the window message-only, so it is never shown or
            // activated and takes no focus.
            let hwnd = unsafe {
                CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    class,
                    PCWSTR::null(),
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

    #[test]
    fn the_desktop_window_reads_as_the_desktop_class_atom() {
        // The desktop window belongs to another process, so this is the
        // cross-process read a dialog of another application needs.
        // SAFETY: GetDesktopWindow takes no arguments and reads no memory of
        // ours.
        let desktop = unsafe { GetDesktopWindow() };
        assert_eq!(class_atom(desktop), DESKTOP_CLASS_ATOM);
        assert!(!is_dialog(desktop));
    }

    #[test]
    fn a_window_of_the_dialog_class_reads_as_a_dialog() {
        let dialog = MessageOnlyWindow::create(w!("#32770"));
        assert_eq!(class_atom(dialog.0), DIALOG_CLASS_ATOM);
        assert!(is_dialog(dialog.0));
    }

    #[test]
    fn a_static_window_and_no_window_are_not_dialogs() {
        let label = MessageOnlyWindow::create(w!("STATIC"));
        assert!(!is_dialog(label.0));
        assert_eq!(class_atom(HWND::default()), 0);
        assert!(!is_dialog(HWND::default()));
    }

    #[test]
    fn a_thread_with_no_focus_window_has_no_focus() {
        // A message-only window is never activated, so the test thread has
        // no active window and no focus.
        let label = MessageOnlyWindow::create(w!("STATIC"));
        assert_eq!(focus_window_of(label.0), Ok(None));
    }

    #[test]
    fn no_window_has_no_thread_whose_focus_could_be_read() {
        // If thread 0, the foreground thread, ever reached GetGUIThreadInfo,
        // this would read the focus of whatever window is in front.
        assert!(focus_window_of(HWND::default()).is_err());
    }

    #[test]
    #[ignore = "manual: start it, then within 5 s click into the File name box of a Save As dialog; it expects the foreground window to read as a dialog and its thread's focus to be a window other than the dialog"]
    fn a_save_as_dialog_reads_as_a_dialog_with_a_focused_control() {
        std::thread::sleep(std::time::Duration::from_secs(5));
        // SAFETY: GetForegroundWindow takes no arguments and reads no memory
        // of ours; with no foreground window it returns a null handle.
        let foreground = unsafe { GetForegroundWindow() };
        assert!(is_dialog(foreground), "the foreground is not a #32770");
        match focus_window_of(foreground) {
            Ok(Some(focus)) => assert_ne!(focus, foreground, "the dialog has the focus itself"),
            other => panic!("no focus window in the dialog's thread: {other:?}"),
        }
    }
}

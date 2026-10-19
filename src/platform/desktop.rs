//! Which desktop has keyboard input (roadmap item 2.3).
//!
//! A lock, the Ctrl+Alt+Del screen and a UAC prompt on the secure desktop
//! move keyboard input to another desktop, whose key-ups the keyboard hook
//! of this desktop never sees ("The hook procedure of a process running on
//! a particular desktop can only receive messages intended for windows
//! created in the same desktop", Microsoft Learn, Desktops). After an
//! `EVENT_SYSTEM_DESKTOPSWITCH` the daemon asks whether its own desktop is
//! the one that receives input now. It asks about its own desktop only:
//! the Winlogon desktop admits a very restricted set of accounts, so the
//! daemon could not open it. Nothing here imports the rest of the crate.

use std::ffi::c_void;

use windows::{
    Win32::{
        Foundation::HANDLE,
        System::{
            StationsAndDesktops::{GetThreadDesktop, GetUserObjectInformationW, UOI_IO},
            Threading::GetCurrentThreadId,
        },
    },
    core::BOOL,
};

/// Whether the desktop of the calling thread is the one that receives user
/// input now: `UOI_IO` of `GetUserObjectInformationW`, "TRUE if the hObj
/// parameter is a handle to the desktop object that is receiving input
/// from the user".
pub fn input_is_on_this_desktop() -> windows::core::Result<bool> {
    // SAFETY: GetCurrentThreadId and GetThreadDesktop take their arguments
    // by value and read no memory of ours. The desktop handle needs no
    // closing: "You do not need to call the CloseDesktop function to close
    // the returned handle" (GetThreadDesktop).
    let desktop = unsafe { GetThreadDesktop(GetCurrentThreadId()) }?;
    let mut value = BOOL(0);
    // SAFETY: `value` is a local BOOL that outlives the call, and the
    // length passed is its size, so the call writes within it; the needed
    // length is not asked for. The desktop handle is the thread's own,
    // open while the thread lives.
    unsafe {
        GetUserObjectInformationW(
            HANDLE(desktop.0),
            UOI_IO,
            Some(&mut value as *mut BOOL as *mut c_void),
            size_of::<BOOL>() as u32,
            None,
        )
    }?;
    Ok(value.as_bool())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "manual: run it at the unlocked desktop (not over Remote Desktop with the session locked); it expects Ok(true): this desktop receives keyboard input"]
    fn input_is_on_this_desktop_while_a_human_uses_it() {
        assert_eq!(input_is_on_this_desktop(), Ok(true));
    }
}

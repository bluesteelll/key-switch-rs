//! A hold of the keyboard callback for the hook watchdog's manual check
//! (roadmap item 2.4, cut D10): debug builds only.
//!
//! With `KS_DEBUG_HOOK_HOLD` set to a key name at startup, such as
//! `ScrollLock`, the keyboard callback sleeps 1.5 s at each key-down of that
//! key, past the largest `LowLevelHooksTimeout` of 1 s, so that Windows
//! removes the hook and the watchdog has it reinstalled. The module is
//! declared under `#[cfg(debug_assertions)]`, and so is every call of it:
//! the release build, the elevated daemon, has no code, no variable and no
//! per-event load for it. `cargo test` never runs `App::run`, which arms it,
//! so under test nothing holds.

use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;

use windows::Win32::Foundation::WPARAM;
use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;
use windows::Win32::UI::WindowsAndMessaging::{WM_KEYDOWN, WM_SYSKEYDOWN};

use crate::data::vk_name::{parse_vk, vk_name};

/// The environment variable that names the key to hold at.
pub(crate) const ENV_VAR: &str = "KS_DEBUG_HOOK_HOLD";

/// How long the callback holds: past the largest `LowLevelHooksTimeout`.
pub(crate) const HOLD: Duration = Duration::from_millis(1500);

/// The virtual key to hold at, 0 for none. Stored once by the main thread
/// before the hooks are installed (`Release`), loaded by the callback on the
/// same thread (`Acquire`); it publishes no other data.
static ARMED: AtomicU16 = AtomicU16::new(0);

/// Arms the hold from `KS_DEBUG_HOOK_HOLD`, if it is set, and says so. Main
/// thread, before the hooks are installed.
pub(crate) fn arm_from_env() {
    let Some(text) = std::env::var_os(ENV_VAR) else {
        return;
    };
    let text = text.to_string_lossy();
    match parse(&text) {
        Some(vk) => {
            ARMED.store(vk.0, Ordering::Release);
            log::warn!(
                "debug build: the keyboard callback holds 1.5 s at each {} key-down ({ENV_VAR})",
                vk_name(vk)
            );
        }
        None => log::warn!("{ENV_VAR} names no key ({text:?}); no hold"),
    }
}

/// The key `text` names, as a config names it.
pub(crate) fn parse(text: &str) -> Option<VIRTUAL_KEY> {
    parse_vk(text)
}

/// Whether the event `wparam` of `vk_code` holds the callback, with
/// `armed` the key armed (0 for none): a key-down or a sys-key-down of it.
pub(crate) fn should_hold(armed: u16, wparam: WPARAM, vk_code: u32) -> bool {
    armed != 0
        && vk_code == u32::from(armed)
        && matches!(u32::try_from(wparam.0), Ok(WM_KEYDOWN | WM_SYSKEYDOWN))
}

/// Holds the calling keyboard callback for [`HOLD`] if the event asks for
/// it. Debug builds only, for the manual check: a release build has no such
/// call.
pub(crate) fn hold_if_asked(wparam: WPARAM, vk_code: u32) {
    if should_hold(ARMED.load(Ordering::Acquire), wparam, vk_code) {
        std::thread::sleep(HOLD);
    }
}

#[cfg(test)]
mod tests {
    use windows::Win32::Foundation::WPARAM;
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_A, VK_SCROLL};
    use windows::Win32::UI::WindowsAndMessaging::{
        WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
    };

    use super::*;

    #[test]
    fn the_hold_key_is_named_like_a_config_key() {
        assert_eq!(parse("ScrollLock"), Some(VK_SCROLL));
        assert_eq!(parse("nope"), None);
        assert_eq!(parse(""), None);
    }

    #[test]
    fn only_a_key_down_of_the_armed_key_holds() {
        let scroll = u32::from(VK_SCROLL.0);
        let message = |msg: u32| WPARAM(msg as usize);
        assert!(should_hold(VK_SCROLL.0, message(WM_KEYDOWN), scroll));
        assert!(should_hold(VK_SCROLL.0, message(WM_SYSKEYDOWN), scroll));
        assert!(!should_hold(VK_SCROLL.0, message(WM_KEYUP), scroll));
        assert!(!should_hold(VK_SCROLL.0, message(WM_SYSKEYUP), scroll));
        assert!(!should_hold(
            VK_SCROLL.0,
            message(WM_KEYDOWN),
            u32::from(VK_A.0)
        ));
        assert!(
            !should_hold(0, message(WM_KEYDOWN), 0),
            "no key armed: nothing holds"
        );
    }
}

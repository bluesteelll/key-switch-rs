//! The engine's view of everything outside it.

use std::time::Instant;

use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

use crate::data::condition::ForegroundInfo;

/// Everything the engine needs to know about the world outside the event it
/// is deciding: the clock, the operating system's own key state and the
/// foreground window.
///
/// The hook adapter implements it with the real Win32 calls
/// (`crate::hook::keyboard_hook_callback`). The test harness in
/// `crate::engine::testing` implements it with a fake clock, a modelled OS
/// key state and a configurable foreground app and title. The engine never
/// reads time or OS state any other way, so a scenario is deterministic.
///
/// Implementations may cache per event: the adapter creates a fresh `Env`
/// for every hook invocation and captures the foreground window at most
/// once, on the first `foreground_*` call.
pub trait Env {
    /// Monotonic time of the event being decided. The engine calls it once
    /// per event; every deadline it stores is derived from it.
    fn now(&self) -> Instant;

    /// Whether the operating system currently reports `vk` as held (its
    /// asynchronous key state). For the generic modifier codes (`VK_SHIFT`,
    /// `VK_CONTROL`, `VK_MENU`) that means "either side is down".
    ///
    /// The OS state only changes for events the hook lets through, and the
    /// state of the key being delivered lags until the hook returns. The
    /// engine therefore asks only about modifiers, and never about the key
    /// of the current event.
    fn os_key_down(&self, vk: VIRTUAL_KEY) -> bool;

    /// Base name of the foreground window's executable (`"code.exe"`), or
    /// `None` if there is no foreground window or it cannot be inspected.
    fn foreground_app(&self) -> Option<&str>;

    /// Title of the foreground window, or `None` if there is none.
    fn foreground_title(&self) -> Option<&str>;
}

/// Lets a `when:` [`Condition`](crate::data::condition::Condition) read the
/// foreground through an [`Env`]. Nothing is fetched until a condition other
/// than `Always` asks.
pub(super) struct EnvForeground<'a>(pub(super) &'a dyn Env);

impl ForegroundInfo for EnvForeground<'_> {
    fn app(&self) -> Option<&str> {
        self.0.foreground_app()
    }

    fn title(&self) -> Option<&str> {
        self.0.foreground_title()
    }
}

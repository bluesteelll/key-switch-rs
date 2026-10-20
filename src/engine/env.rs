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
/// Implementations may cache: the adapter creates a fresh `Env` for every
/// hook invocation, answers `foreground_app` from the main thread's
/// per-window cache, which the foreground event fills (roadmap item 2.3),
/// and reads the title at most once per event, on the first
/// `foreground_title` call.
pub trait Env {
    /// Monotonic time of the event being decided. The engine calls it once
    /// per event; every deadline it stores is derived from it.
    ///
    /// The hook adapter answers with `Instant::now()`, on the main thread,
    /// which also takes a timer only once `Instant::now()` has reached its
    /// due time, and no lock is involved. Events and timers are thus decided
    /// in the order of this one clock.
    fn now(&self) -> Instant;

    /// Whether the operating system currently reports `vk` as held (its
    /// asynchronous key state). For the generic modifier codes (`VK_SHIFT`,
    /// `VK_CONTROL`, `VK_MENU`) that means "either side is down".
    ///
    /// The OS state only changes for events the hook lets through, and the
    /// state of the key being delivered lags until the hook returns. The
    /// engine therefore never asks about the key of the current event,
    /// except at a key-up it would swallow, where the state before the
    /// event is exactly the question: down means Windows holds a key-down
    /// of it that the engine did not swallow, so the key-up passes (roadmap
    /// item 2.3). A key-down asks about each modifier held for the engine
    /// whose key-down it did not swallow, and a key-up it swallows about
    /// each such Alt and Win, to drop those Windows has up (roadmap item
    /// 2.5); the state of a key whose earlier event Windows has applied is
    /// the question there. The main thread's read between key events asks
    /// about the sided modifiers, leaving out every modifier whose key-down
    /// it swallowed and every one an event touched since the last read, and
    /// about a generic code only while the engine holds it. An auto-blocker asks,
    /// without overwriting anything, about the held keys outside its
    /// hotkey, to learn whether Windows would see more than the hotkey. The
    /// reset of `swch off` and `swch on`, which has no event, asks about
    /// every keyboard code but the generic modifier codes to learn which
    /// keys Windows has down; the reset of keyboard input leaving this
    /// desktop asks nothing, because the OS state is no witness then.
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

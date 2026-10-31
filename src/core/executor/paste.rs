//! The `Paste` action and macro step on the action executor (owner request
//! 2026-10-30, roadmap new feature 9).
//!
//! What a paste needs besides the clipboard (`platform::clipboard`) goes
//! through a [`Desk`]: the foreground window, whether a window answers,
//! whether a modifier key is down, and the paste keystroke. The daemon's is
//! the executor's `Win32Desk`, whose Win32 calls sit in the `win32` module
//! below, which `#[cfg(not(test))]` keeps out of the test build: under test
//! it panics at every call, so no test sends a key or probes a window of the
//! machine that runs it. The tests hand in a fake desk instead.

use std::time::{Duration, Instant};

use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

use super::TargetWindow;
use crate::data::paste::PasteSpec;
use crate::data::sequence::{self, ComboSent};
use crate::platform::clipboard::{
    self, CF_UNICODETEXT, Clipboard, Marker, OwnerWindow, Payload, Read, Zeroed,
};

/// What a paste reads from and sends to the desktop, besides the
/// clipboard. Every method runs on the action executor.
pub trait Desk {
    /// The foreground window now, or `NONE`.
    fn foreground(&self) -> TargetWindow;
    /// Whether the thread of `window` answers a `WM_NULL` within `within`.
    fn responds(&self, window: isize, within: Duration) -> Probe;
    /// Whether Windows has a Shift, Ctrl, Alt or Win key down, either one
    /// of each pair.
    fn modifier_down(&self) -> bool;
    /// Sends the paste keystroke `keys` in one batch (`press_combo`), each
    /// event with the sentinel.
    fn press(&self, keys: &[VIRTUAL_KEY]) -> ComboSent;
    /// Sends the key-ups a batch of `keys` cut short after `sent` events
    /// still owes, and returns how many went out.
    fn release_after(&self, keys: &[VIRTUAL_KEY], sent: u32) -> u32;
}

/// What a [`Desk::responds`] probe found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Probe {
    /// The window's thread answered at once.
    Answered,
    /// It answered, but only after a noticeable wait: it was busy, and the
    /// input queued for it is handled only now (cut critique M1).
    Late,
    /// It did not answer in time, or it looks hung.
    NoAnswer,
    /// The window no longer exists (cut critique I1): nothing will read
    /// the clipboard for it.
    Gone,
}

/// The clipboard and the desk a paste works through.
pub struct Ports<'a> {
    pub clipboard: &'a dyn Clipboard,
    pub desk: &'a dyn Desk,
}

/// The daemon's desk: the Win32 calls of the `win32` module outside tests,
/// a panic inside them. Crate-private, so that no doctest or integration
/// test can reach a keystroke through it (cut critique M6).
pub(crate) struct Win32Desk;

/// The desk the daemon uses.
pub(crate) const WIN32_DESK: Win32Desk = Win32Desk;

#[cfg(not(test))]
mod win32 {
    use std::time::{Duration, Instant};

    use windows::Win32::{
        Foundation::{HWND, LPARAM, WPARAM},
        UI::{
            Input::KeyboardAndMouse::*,
            WindowsAndMessaging::{
                IsWindow, SMTO_ABORTIFHUNG, SMTO_BLOCK, SendMessageTimeoutW, WM_NULL,
            },
        },
    };

    use super::{Desk, Probe, Win32Desk};
    use crate::core::executor::TargetWindow;
    use crate::data::sequence::{self, ComboSent};

    /// The modifier keys whose state the wait reads: both keys of each
    /// pair, as Windows reports them.
    const SIDED_MODIFIERS: [VIRTUAL_KEY; 8] = [
        VK_LSHIFT,
        VK_RSHIFT,
        VK_LCONTROL,
        VK_RCONTROL,
        VK_LMENU,
        VK_RMENU,
        VK_LWIN,
        VK_RWIN,
    ];

    /// An answer that took at least this long counts as a busy thread's: a
    /// responsive one answers a `WM_NULL` in well under a millisecond.
    const LATE_ANSWER: Duration = Duration::from_millis(5);

    fn exists(window: HWND) -> bool {
        // SAFETY: IsWindow takes the handle by value and reads no memory of
        // ours; a handle that names no window gives FALSE.
        unsafe { IsWindow(Some(window)) }.as_bool()
    }

    impl Desk for Win32Desk {
        fn foreground(&self) -> TargetWindow {
            TargetWindow::foreground()
        }

        fn responds(&self, window: isize, within: Duration) -> Probe {
            let window = HWND(window as *mut core::ffi::c_void);
            if !exists(window) {
                return Probe::Gone;
            }
            let timeout = u32::try_from(within.as_millis()).unwrap_or(u32::MAX);
            let begun = Instant::now();
            // SAFETY: WM_NULL carries no pointer and no result is asked for,
            // so the call reads and writes no memory of ours; SMTO_BLOCK
            // keeps this thread from handling other sent messages while it
            // waits, SMTO_ABORTIFHUNG returns at once for a thread Windows
            // takes for hung, and a stale handle only makes it fail.
            let answered = unsafe {
                SendMessageTimeoutW(
                    window,
                    WM_NULL,
                    WPARAM(0),
                    LPARAM(0),
                    SMTO_ABORTIFHUNG | SMTO_BLOCK,
                    timeout,
                    None,
                )
            };
            if answered.0 == 0 {
                // "If the function fails or times out, the return value is
                // 0": a window that went away meanwhile is gone, not hung.
                return if exists(window) {
                    Probe::NoAnswer
                } else {
                    Probe::Gone
                };
            }
            if begun.elapsed() >= LATE_ANSWER {
                Probe::Late
            } else {
                Probe::Answered
            }
        }

        fn modifier_down(&self) -> bool {
            SIDED_MODIFIERS.iter().any(|&key| {
                // SAFETY: GetAsyncKeyState takes the key code by value,
                // writes no memory of ours and may be called from any
                // thread; a failure (another desktop, UIPI) reads as up.
                let state = unsafe { GetAsyncKeyState(i32::from(key.0)) };
                (state as u16) & 0x8000 != 0
            })
        }

        fn press(&self, keys: &[VIRTUAL_KEY]) -> ComboSent {
            sequence::press_combo(keys)
        }

        fn release_after(&self, keys: &[VIRTUAL_KEY], sent: u32) -> u32 {
            sequence::release_unreleased(keys, sent)
        }
    }
}

/// What every call of the daemon's desk says under test.
#[cfg(test)]
const UNDER_TEST: &str =
    "a test reached the daemon's desk, which no test build calls (lane feat-clipboard-paste)";

/// Under test no call reaches Windows: a test that gets here has a defect,
/// and fails at the panic instead of pressing a key on the machine that
/// runs it.
#[cfg(test)]
impl Desk for Win32Desk {
    fn foreground(&self) -> TargetWindow {
        unreachable!("{UNDER_TEST}")
    }

    fn responds(&self, _window: isize, _within: Duration) -> Probe {
        unreachable!("{UNDER_TEST}")
    }

    fn modifier_down(&self) -> bool {
        unreachable!("{UNDER_TEST}")
    }

    fn press(&self, _keys: &[VIRTUAL_KEY]) -> ComboSent {
        unreachable!("{UNDER_TEST}")
    }

    fn release_after(&self, _keys: &[VIRTUAL_KEY], _sent: u32) -> u32 {
        unreachable!("{UNDER_TEST}")
    }
}

/// How long a Paste waits for the restore of the one before it.
pub const CHAIN_WAIT_LIMIT: Duration = Duration::from_millis(1_000);
/// How often a Paste reads the modifiers while one is held.
pub const MODIFIER_POLL: Duration = Duration::from_millis(5);
/// How long a Paste waits for the modifiers to be released.
pub const MODIFIER_LIMIT: Duration = Duration::from_millis(3_000);
/// How long the clipboard's owner may take to answer before a paste.
pub const OWNER_PROBE: Duration = Duration::from_millis(100);
/// The waits between the tries to open a busy clipboard for a paste.
pub const OPEN_BACKOFF: [Duration; 5] = [
    Duration::from_millis(5),
    Duration::from_millis(10),
    Duration::from_millis(20),
    Duration::from_millis(40),
    Duration::from_millis(80),
];
/// After this long no other format than the text is read for the save.
pub const SAVE_BUDGET: Duration = Duration::from_millis(50);
/// The most bytes a save keeps.
pub const SAVE_CAP: usize = 64 << 20;
/// How long the restore waits at most for a target that does not answer.
pub const HUNG_LIMIT: Duration = Duration::from_millis(5_000);
/// How long the target may take to answer at the restore's deadline.
pub const TARGET_PROBE: Duration = Duration::from_millis(20);
/// The wait before the next probe of a target that did not answer.
pub const TARGET_RETRY: Duration = Duration::from_millis(250);
/// The wait after a busy target answered, for the keystroke queued for it.
pub const SETTLE: Duration = Duration::from_millis(100);
/// The wait between the tries to open a busy clipboard for the restore.
pub const RESTORE_RETRY: Duration = Duration::from_millis(25);
/// How long the restore tries to open a busy clipboard.
pub const RESTORE_RETRY_LIMIT: Duration = Duration::from_millis(1_000);
/// The wait between the tries of the restore when the executor stops.
pub const CLOSING_RETRY: Duration = Duration::from_millis(10);
/// How long the executor's stop waits for a busy clipboard.
pub const CLOSING_LIMIT: Duration = Duration::from_millis(250);

/// The most bytes read of a `CanIncludeInClipboardHistory` marker: a DWORD.
const MARKER_READ_LIMIT: usize = 64;
/// The data of every marker the daemon writes: a DWORD 0. A NULL handle
/// would ask for delayed rendering instead.
const MARKER_DATA: [u8; 4] = [0; 4];

/// What a Paste that waits keeps between its tries, in its run.
#[derive(Debug)]
pub(crate) struct Attempt {
    /// The foreground window when the paste started: the keystroke goes to
    /// it or nowhere.
    foreground: TargetWindow,
    /// When it began to wait for the restore of the paste before it.
    chain_since: Option<Instant>,
    /// Whether it went ahead at that wait's limit, so that L20 is logged
    /// once.
    went_ahead: bool,
    /// When it began to wait for the modifiers' release.
    modifiers_since: Option<Instant>,
    /// Whether the clipboard's owner was probed.
    prechecked: bool,
    /// How many opens failed, and when the first one did.
    open_tries: u8,
    open_since: Option<Instant>,
}

impl Attempt {
    fn new(foreground: TargetWindow) -> Self {
        Self {
            foreground,
            chain_since: None,
            went_ahead: false,
            modifiers_since: None,
            prechecked: false,
            open_tries: 0,
            open_since: None,
        }
    }

    /// The wait after an open that failed at `now`, until the retries of
    /// `OPEN_BACKOFF` are spent.
    fn open_failed(&mut self, now: Instant) -> Step {
        let since = *self.open_since.get_or_insert(now);
        if let Some(wait) = OPEN_BACKOFF.get(usize::from(self.open_tries)) {
            self.open_tries += 1;
            return Step::WaitUntil(now + *wait);
        }
        log::warn!(
            "paste: another program kept the clipboard open for {} ms; the paste was skipped",
            now.duration_since(since).as_millis()
        );
        Step::Done
    }
}

/// What became of a try of a Paste.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Step {
    /// The paste is over: sent, or skipped and logged.
    Done,
    /// It waits; try again once `now` reaches this.
    WaitUntil(Instant),
}

/// What the restore at the executor's stop did.
// Only the restore at the stop makes `RetryAt`; this allow goes with it.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Closing {
    /// Nothing is left to put back.
    Done,
    /// The clipboard was busy: try again then.
    RetryAt(Instant),
}

/// The ids of the marker formats, in the order of `Marker::ALL`, registered
/// once per daemon; `None` for one whose registration failed, which is left
/// out.
#[derive(Debug, Clone, Copy)]
struct Markers([Option<u32>; 5]);

impl Markers {
    fn register(clipboard: &dyn Clipboard) -> Self {
        Self(Marker::ALL.map(|marker| {
            let id = clipboard.register(marker);
            if id.is_none() {
                log::warn!(
                    "paste: could not register the clipboard format {}; the text may appear in the clipboard history",
                    marker.name()
                );
            }
            id
        }))
    }

    /// The ids a phrase carries: every marker but the daemon's own.
    fn phrase(&self) -> impl Iterator<Item = u32> {
        Marker::ALL
            .into_iter()
            .zip(self.0)
            .filter(|(marker, _)| *marker != Marker::Restored)
            .filter_map(|(_, id)| id)
    }

    /// The ids a restored copy carries: every marker.
    fn restored(&self) -> impl Iterator<Item = u32> {
        self.0.into_iter().flatten()
    }
}

/// A format of the clipboard as the save copied it.
#[derive(Debug)]
struct SavedFormat {
    format: u32,
    /// Its data, whose `Debug` shows the length only.
    bytes: Zeroed,
}

/// What the restore puts back.
#[derive(Debug)]
enum Original {
    /// The formats the save copied, in enumeration order.
    Saved(Vec<SavedFormat>),
    /// Content marked private (owner answer): the restore empties the
    /// clipboard.
    Private,
    /// An empty clipboard, or one of which nothing could be copied: the
    /// restore empties it.
    Empty,
}

/// The restore that waits, from the daemon's `EmptyClipboard` on: whatever
/// fails or unwinds after that point, the original goes back (research
/// critique M1).
#[derive(Debug)]
struct Pending {
    /// The window that emptied the clipboard, and owns it while it holds
    /// the daemon's phrase.
    window: OwnerWindow,
    original: Original,
    /// The formats of the daemon's phrase: the text and the markers.
    written: Vec<u32>,
    /// When the restore is decided.
    restore_at: Instant,
    /// When it is due whatever the target does.
    hard_limit: Instant,
    /// The window the keystroke went to; `None` until the keystroke went
    /// out, and then the restore is due at `restore_at`.
    target: Option<isize>,
    /// Whether a probe found the target busy.
    seen_busy: bool,
    /// Whether the wait after a busy target answered was given.
    settled: bool,
    /// What the last probe of the target found.
    last_probe: Option<Probe>,
    /// Whether the restore is due.
    due: bool,
    /// When the restore first found the clipboard busy, and when it tries
    /// again.
    retry_since: Option<Instant>,
    next_try: Option<Instant>,
    /// The clipboard's sequence number after the phrase was set.
    sequence: u32,
}

impl Pending {
    fn new(window: OwnerWindow, original: Original, written: Vec<u32>, now: Instant) -> Self {
        Self {
            window,
            original,
            written,
            restore_at: now,
            hard_limit: now,
            target: None,
            seen_busy: false,
            settled: false,
            last_probe: None,
            due: false,
            retry_since: None,
            next_try: None,
            sequence: 0,
        }
    }

    /// Starts over for a phrase set at `now`: the restore is due at the
    /// next poll unless a keystroke sets its deadline.
    fn renew(&mut self, now: Instant) {
        self.restore_at = now;
        self.hard_limit = now;
        self.target = None;
        self.seen_busy = false;
        self.settled = false;
        self.last_probe = None;
        self.due = false;
        self.retry_since = None;
        self.next_try = None;
    }
}

/// What a try at the restore found.
enum Restore {
    /// The original is back, or the clipboard emptied for it.
    PutBack,
    /// Another program changed the clipboard: nothing was emptied.
    Changed,
    /// Another program held the clipboard open.
    Busy,
}

/// The clipboard, open by the session: dropping it closes the clipboard on
/// every path, unwinding included.
struct Opened<'a>(&'a dyn Clipboard);

impl<'a> Opened<'a> {
    fn open(clipboard: &'a dyn Clipboard, window: OwnerWindow) -> Option<Self> {
        clipboard.open(window).ok().map(|()| Self(clipboard))
    }
}

impl Drop for Opened<'_> {
    fn drop(&mut self) {
        if let Err(error) = self.0.close() {
            log::warn!("paste: could not close the clipboard ({error})");
        }
    }
}

/// The executor's one paste session: the clipboard window and the restore
/// that waits, if any. It lives in the runs, outside the calls that can
/// unwind, on the executor thread.
#[derive(Debug, Default)]
pub(crate) struct PasteSession {
    /// The window the next fresh paste empties the clipboard with.
    window: Option<OwnerWindow>,
    /// The window of the session that ended last, which may still own the
    /// clipboard.
    last_window: Option<OwnerWindow>,
    pending: Option<Pending>,
    markers: Option<Markers>,
}

impl PasteSession {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Tries the Paste of `spec`, whose state between tries is `attempt`
    /// (D6): it goes as far as it can, and returns when it must wait or is
    /// over.
    pub(crate) fn attempt(
        &mut self,
        attempt: &mut Option<Attempt>,
        spec: &PasteSpec,
        ports: &Ports<'_>,
        now: &dyn Fn() -> Instant,
        live: &dyn Fn() -> bool,
    ) -> Step {
        let step = self.advance(attempt, spec, ports, now, live);
        if step == Step::Done {
            *attempt = None;
        }
        step
    }

    fn advance(
        &mut self,
        attempt: &mut Option<Attempt>,
        spec: &PasteSpec,
        ports: &Ports<'_>,
        now: &dyn Fn() -> Instant,
        live: &dyn Fn() -> bool,
    ) -> Step {
        let state = match attempt {
            Some(state) => state,
            None => {
                let foreground = ports.desk.foreground();
                if foreground == TargetWindow::NONE {
                    log::warn!("paste: no foreground window; the paste was skipped");
                    return Step::Done;
                }
                attempt.insert(Attempt::new(foreground))
            }
        };
        if let Some(pending) = &self.pending
            && !pending.due
            && !state.went_ahead
        {
            let at = now();
            let limit = *state.chain_since.get_or_insert(at) + CHAIN_WAIT_LIMIT;
            if at < limit {
                return Step::WaitUntil(pending.restore_at.min(limit));
            }
            log::info!(
                "paste: a paste went ahead after waiting 1 s for the previous one's restore"
            );
            state.went_ahead = true;
        }
        if ports.desk.modifier_down() {
            let at = now();
            let since = *state.modifiers_since.get_or_insert(at);
            if at.duration_since(since) >= MODIFIER_LIMIT {
                log::warn!("paste: a modifier key was still held after 3 s; the paste was skipped");
                return Step::Done;
            }
            return Step::WaitUntil(at + MODIFIER_POLL);
        }
        if !live() {
            return Step::Done;
        }
        if ports.desk.foreground() != state.foreground {
            log::warn!(
                "paste: the foreground window changed before the paste; the paste was skipped"
            );
            return Step::Done;
        }
        let clipboard = ports.clipboard;
        let markers = *self
            .markers
            .get_or_insert_with(|| Markers::register(clipboard));
        loop {
            let window = match &self.pending {
                Some(pending) => pending.window,
                None => {
                    let Some(window) = self.idle_window(clipboard) else {
                        return Step::Done;
                    };
                    if !state.prechecked {
                        state.prechecked = true;
                        if self.owner_hangs(ports) {
                            log::warn!(
                                "paste: the program that owns the clipboard did not answer within 100 ms; the paste was skipped"
                            );
                            return Step::Done;
                        }
                    }
                    window
                }
            };
            let Some(opened) = Opened::open(clipboard, window) else {
                return state.open_failed(now());
            };
            if let Some(pending) = &self.pending {
                // A chained paste takes over the phrase of the one before
                // it only while the clipboard still holds that phrase.
                if !still_ours(clipboard, pending) {
                    log::info!(
                        "paste: the clipboard changed after the paste; the previous content was not put back"
                    );
                    drop(opened);
                    self.end_session(clipboard);
                    continue;
                }
                if let Err(error) = clipboard.empty() {
                    log::warn!(
                        "paste: could not empty the clipboard ({error}); the paste was skipped"
                    );
                    return Step::Done;
                }
            } else {
                let original = save(clipboard, now);
                if let Err(error) = clipboard.empty() {
                    log::warn!(
                        "paste: could not empty the clipboard ({error}); the paste was skipped"
                    );
                    return Step::Done;
                }
                let written = std::iter::once(CF_UNICODETEXT)
                    .chain(markers.phrase())
                    .collect();
                self.window = None;
                self.pending = Some(Pending::new(window, original, written, now()));
            }
            return self.set_phrase(opened, state, spec, ports, now, live);
        }
    }

    /// Puts the phrase and its markers on the clipboard this session has
    /// open and just emptied, closes it, and sends the keystroke if nothing
    /// changed meanwhile (D6 rows 9 to 13).
    fn set_phrase(
        &mut self,
        opened: Opened<'_>,
        state: &Attempt,
        spec: &PasteSpec,
        ports: &Ports<'_>,
        now: &dyn Fn() -> Instant,
        live: &dyn Fn() -> bool,
    ) -> Step {
        let clipboard = ports.clipboard;
        let markers = self
            .markers
            .expect("invariant: the markers are registered before the open");
        let pending = self
            .pending
            .as_mut()
            .expect("invariant: the clipboard was emptied for this paste");
        pending.renew(now());
        if let Err(error) = clipboard.write(CF_UNICODETEXT, Payload::Units(spec.units())) {
            put_back(clipboard, pending, Some(&markers));
            drop(opened);
            self.end_session(clipboard);
            log::warn!(
                "paste: could not put the text on the clipboard ({error}); the clipboard was put back and the paste skipped"
            );
            return Step::Done;
        }
        let mut marked = true;
        for id in markers.phrase() {
            if let Err(error) = clipboard.write(id, Payload::Bytes(&MARKER_DATA))
                && marked
            {
                marked = false;
                log::warn!(
                    "paste: could not mark the text as private ({error}); it may appear in the clipboard history"
                );
            }
        }
        drop(opened);
        pending.sequence = clipboard.sequence_number();
        if !live() || ports.desk.foreground() != state.foreground || ports.desk.modifier_down() {
            log::info!(
                "paste: the foreground window, the modifiers or the remapping switch changed after the clipboard was set; it was put back and the paste skipped"
            );
            return Step::Done;
        }
        let keys = spec.keys();
        let ComboSent { sent, of } = ports.desk.press(keys);
        let keystroke = now();
        if sent == 0 {
            log::warn!("paste: the paste keystroke was not sent; the clipboard is put back");
            return Step::Done;
        }
        if sent < of {
            ports.desk.release_after(keys, sent);
            log::warn!(
                "paste: the paste keystroke was cut short ({sent} of {of} events); the missing key releases were sent"
            );
            // The batch sends the modifiers' key-downs first: if no other
            // key went down, nothing was pasted and nothing will read the
            // phrase.
            let modifiers = keys
                .iter()
                .filter(|&&key| sequence::is_modifier(key))
                .count();
            if usize::try_from(sent).is_ok_and(|sent| sent <= modifiers) {
                return Step::Done;
            }
        }
        pending.restore_at = keystroke + spec.restore_delay();
        pending.hard_limit = keystroke + spec.restore_delay().max(HUNG_LIMIT);
        pending.target = Some(state.foreground.0);
        Step::Done
    }

    /// The idle window, made if there is none; `None`, logged, if it
    /// cannot be.
    fn idle_window(&mut self, clipboard: &dyn Clipboard) -> Option<OwnerWindow> {
        if self.window.is_none() {
            match clipboard.create_window() {
                Ok(window) => self.window = Some(window),
                Err(error) => log::error!(
                    "paste: could not create the clipboard window ({error}); the paste was skipped"
                ),
            }
        }
        self.window
    }

    /// Whether the program that owns the clipboard does not answer: an
    /// `OpenClipboard` would wait for its delayed renderings. The daemon's
    /// own windows are not asked.
    fn owner_hangs(&self, ports: &Ports<'_>) -> bool {
        let Some(owner) = ports.clipboard.owner() else {
            return false;
        };
        let ours = |window: Option<OwnerWindow>| window.is_some_and(|window| window.raw() == owner);
        !ours(self.window)
            && !ours(self.last_window)
            && ports.desk.responds(owner, OWNER_PROBE) == Probe::NoAnswer
    }

    /// Decides whether the restore is due (D10): at its deadline, once the
    /// target answers, at once if it is gone, at the hard limit whatever it
    /// does. `true` if it asked a port.
    pub(crate) fn tick(&mut self, ports: &Ports<'_>, now: Instant) -> bool {
        let Some(pending) = &mut self.pending else {
            return false;
        };
        if pending.due || now < pending.restore_at {
            return false;
        }
        let Some(target) = pending.target else {
            pending.due = true;
            return false;
        };
        if now >= pending.hard_limit {
            if pending.last_probe == Some(Probe::NoAnswer) {
                log::warn!(
                    "paste: the target window did not answer for 5 s; the clipboard was put back anyway"
                );
            }
            pending.due = true;
            return false;
        }
        let probe = ports.desk.responds(target, TARGET_PROBE);
        pending.last_probe = Some(probe);
        match probe {
            Probe::Gone => pending.due = true,
            Probe::NoAnswer => {
                pending.seen_busy = true;
                pending.restore_at = (now + TARGET_RETRY).min(pending.hard_limit);
            }
            Probe::Answered | Probe::Late => {
                pending.seen_busy |= probe == Probe::Late;
                // A thread answers a sent message before it handles the
                // input queued for it, so a busy target may not have read
                // the phrase yet.
                if pending.seen_busy && !pending.settled {
                    pending.settled = true;
                    pending.restore_at = (now + SETTLE).min(pending.hard_limit);
                } else {
                    pending.due = true;
                }
            }
        }
        true
    }

    /// Puts the clipboard back if the restore is due, and retries a busy
    /// clipboard every `RESTORE_RETRY` for `RESTORE_RETRY_LIMIT`. `true` if
    /// it asked a port.
    pub(crate) fn restore_if_due(&mut self, ports: &Ports<'_>, now: Instant) -> bool {
        let Some(pending) = &mut self.pending else {
            return false;
        };
        if !pending.due {
            if pending.target.is_some() || now < pending.restore_at {
                return false;
            }
            pending.due = true;
        }
        if pending.next_try.is_some_and(|at| now < at) {
            return false;
        }
        if let Restore::Busy = self.try_restore(ports.clipboard) {
            let pending = self
                .pending
                .as_mut()
                .expect("invariant: a busy restore keeps its session");
            let since = *pending.retry_since.get_or_insert(now);
            if now.duration_since(since) >= RESTORE_RETRY_LIMIT {
                log::warn!(
                    "paste: another program kept the clipboard open for 1 s; the previous content was not put back"
                );
                self.end_session(ports.clipboard);
            } else {
                pending.next_try = Some(now + RESTORE_RETRY);
            }
        }
        true
    }

    /// One try at the restore: under the session's own open, the original
    /// goes back if the clipboard still holds the phrase. The session ends
    /// unless the clipboard was busy.
    fn try_restore(&mut self, clipboard: &dyn Clipboard) -> Restore {
        let Some(pending) = &self.pending else {
            return Restore::PutBack;
        };
        let sequence = clipboard.sequence_number();
        let Some(opened) = Opened::open(clipboard, pending.window) else {
            return Restore::Busy;
        };
        // Checked while the clipboard is open, so that nobody can change it
        // between the check and the `EmptyClipboard` (research critique I1).
        let restore = if still_ours(clipboard, pending) {
            if sequence != pending.sequence {
                log::debug!(
                    "paste: the clipboard sequence number moved by {} while the daemon owned it",
                    sequence.wrapping_sub(pending.sequence)
                );
            }
            put_back(clipboard, pending, self.markers.as_ref());
            Restore::PutBack
        } else {
            log::info!(
                "paste: the clipboard changed after the paste; the previous content was not put back"
            );
            Restore::Changed
        };
        drop(opened);
        self.end_session(clipboard);
        restore
    }

    /// Ends the session: its saved copy is dropped, which wipes it, its
    /// window destroyed, and the idle window for the next paste made. A
    /// window that cannot be made is tried again, and logged, by the next
    /// paste.
    fn end_session(&mut self, clipboard: &dyn Clipboard) {
        if let Some(pending) = self.pending.take() {
            clipboard.destroy_window(pending.window);
            self.last_window = Some(pending.window);
        }
        if self.window.is_none() {
            self.window = clipboard.create_window().ok();
        }
    }

    /// When the session must be polled again, if a restore waits.
    pub(crate) fn next(&self) -> Option<Instant> {
        let pending = self.pending.as_ref()?;
        Some(match pending.next_try {
            Some(at) if pending.due => at,
            _ => pending.restore_at,
        })
    }

    /// Whether a restore waits.
    pub(crate) fn is_pending(&self) -> bool {
        self.pending.is_some()
    }

    /// Ends the session after one of its calls panicked, so that no later
    /// poll panics on it again.
    pub(crate) fn give_up(&mut self, ports: &Ports<'_>) {
        self.end_session(ports.clipboard);
    }

    /// The restore at the executor's stop, tried until `until`.
    pub(crate) fn close(&mut self, _ports: &Ports<'_>, _now: Instant, _until: Instant) -> Closing {
        Closing::Done
    }

    /// The restore now, after the executor's timer failed.
    pub(crate) fn restore_now(&mut self, _ports: &Ports<'_>) {}

    /// The last try at the restore, and the windows' end, as the executor
    /// thread ends.
    pub(crate) fn abandon(&mut self, _ports: &Ports<'_>) {}
}

/// Whether the clipboard, open by the session, still holds the daemon's
/// phrase: owned by the session's window, with no format that another
/// program added without emptying it (research critique M2).
fn still_ours(clipboard: &dyn Clipboard, pending: &Pending) -> bool {
    if clipboard.owner() != Some(pending.window.raw()) {
        return false;
    }
    let mut formats = Vec::new();
    clipboard.formats(&mut formats);
    !clipboard::foreign_format_present(&formats, &pending.written)
}

/// Puts the original of `pending` back on the clipboard, which the session
/// has open and which still holds the phrase: empty it, then the saved
/// formats in order and every marker.
fn put_back(clipboard: &dyn Clipboard, pending: &Pending, markers: Option<&Markers>) {
    if let Err(error) = clipboard.empty() {
        log::warn!(
            "paste: could not empty the clipboard ({error}); the previous content was not put back"
        );
        return;
    }
    let Original::Saved(saved) = &pending.original else {
        return;
    };
    let writes = saved
        .iter()
        .map(|format| (format.format, format.bytes.bytes()))
        .chain(
            markers
                .into_iter()
                .flat_map(Markers::restored)
                .map(|id| (id, &MARKER_DATA[..])),
        );
    let (mut total, mut failed) = (0_usize, 0_usize);
    for (format, bytes) in writes {
        total += 1;
        if clipboard.write(format, Payload::Bytes(bytes)).is_err() {
            failed += 1;
        }
    }
    if failed > 0 {
        log::warn!("paste: {failed} of {total} clipboard formats could not be put back");
    }
}

/// Saves what the clipboard, open by the session, holds (D7): nothing of a
/// private original; else the text first, then the other formats kept in
/// enumeration order while the save is within `SAVE_BUDGET` and
/// `SAVE_CAP`.
fn save(clipboard: &dyn Clipboard, now: &dyn Fn() -> Instant) -> Original {
    let started = now();
    let mut formats = Vec::new();
    clipboard.formats(&mut formats);
    if formats.is_empty() {
        return Original::Empty;
    }
    let names: Vec<Option<String>> = formats
        .iter()
        .map(|&format| registered_name(clipboard, format))
        .collect();
    let marker = |wanted: Marker| {
        formats
            .iter()
            .zip(&names)
            .find(|(_, name)| name.as_deref().and_then(Marker::named) == Some(wanted))
            .map(|(&format, _)| format)
    };
    // A copy this daemon put back carries the markers too.
    let private = marker(Marker::Restored).is_none()
        && (marker(Marker::ExcludeFromMonitoring).is_some()
            || marker(Marker::CanIncludeInHistory)
                .is_some_and(|id| !allows_history(clipboard.read(id, MARKER_READ_LIMIT))));
    if private {
        log::info!("paste: the clipboard held private content; it will be cleared, not put back");
        return Original::Private;
    }
    let kept = clipboard::kept_formats(
        formats
            .iter()
            .copied()
            .zip(names.iter().map(Option::as_deref)),
    );
    let text = kept.iter().position(|&format| format == CF_UNICODETEXT);
    let order = text
        .into_iter()
        .chain((0..kept.len()).filter(|&index| Some(index) != text));
    let mut saved: Vec<(usize, SavedFormat)> = Vec::with_capacity(kept.len());
    let mut total = 0;
    for index in order {
        // The text is read whatever the time: it is what the user pastes
        // most.
        if Some(index) != text && now().duration_since(started) >= SAVE_BUDGET {
            continue;
        }
        let format = kept[index];
        if let Read::Bytes(bytes) = clipboard.read(format, SAVE_CAP.saturating_sub(total)) {
            total += bytes.len();
            saved.push((index, SavedFormat { format, bytes }));
        }
    }
    let lost = kept.len() - saved.len();
    if lost > 0 {
        log::warn!(
            "paste: {lost} of {} clipboard formats will not be restored (too large, too slow or unreadable)",
            kept.len()
        );
    }
    if saved.is_empty() {
        return Original::Empty;
    }
    // A program pastes the first format it knows (cut critique M3).
    saved.sort_by_key(|(index, _)| *index);
    Original::Saved(saved.into_iter().map(|(_, format)| format).collect())
}

/// The name of `format` if it is a registered one whose name can be read.
fn registered_name(clipboard: &dyn Clipboard, format: u32) -> Option<String> {
    if !clipboard::REGISTERED.contains(&format) {
        return None;
    }
    let mut name = String::new();
    clipboard.format_name(format, &mut name).then_some(name)
}

/// Whether a `CanIncludeInClipboardHistory` marker lets the content into
/// the history: a DWORD that is not 0. Shorter or unreadable data does not.
fn allows_history(read: Read) -> bool {
    match read {
        Read::Bytes(bytes) => bytes
            .bytes()
            .get(..4)
            .is_some_and(|dword| dword.iter().any(|&byte| byte != 0)),
        Read::TooBig(_) | Read::NoMemory | Read::Failed => false,
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::sync::{Arc, Mutex, PoisonError};

    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    use super::*;
    use crate::core::executor::testing::{DeskCall, FOREGROUND, FakeDesk};
    use crate::logging::testing::capture;
    use crate::platform::clipboard::{
        CF_LOCALE, CF_UNICODETEXT, Call, Fail, FakeClipboard, FakeClock, Marker, Windows,
    };

    #[test]
    fn the_win32_desk_panics_at_every_call_under_test() {
        let calls: [(&str, &dyn Fn()); 5] = [
            ("foreground", &|| {
                let _ = WIN32_DESK.foreground();
            }),
            ("responds", &|| {
                let _ = WIN32_DESK.responds(0x1234, Duration::from_millis(1));
            }),
            ("modifier_down", &|| {
                let _ = WIN32_DESK.modifier_down();
            }),
            ("press", &|| {
                let _ = WIN32_DESK.press(&[VK_CONTROL, VK_V]);
            }),
            ("release_after", &|| {
                let _ = WIN32_DESK.release_after(&[VK_CONTROL, VK_V], 1);
            }),
        ];
        for (name, call) in calls {
            assert!(
                catch_unwind(AssertUnwindSafe(call)).is_err(),
                "{name} returned under test"
            );
        }
    }

    /// The program that owns the clipboard's original content.
    const OWNER: isize = 0x77;
    /// A program that copies while a paste waits for its restore.
    const OTHER: isize = 0x99;
    const HTML: u32 = 0xC010;
    const RTF: u32 = 0xC011;
    /// The clipboard windows the fake makes, in order.
    const W1: isize = 0x1001;
    const W2: isize = 0x1002;
    const W3: isize = 0x1003;
    /// The locale the fake makes from a text, as Windows does.
    const LOCALE: [u8; 4] = [0x09, 0x04, 0, 0];
    /// The data of a marker: a DWORD 0.
    const ZEROS: &[u8] = &[0; 4];

    /// `text` as `CF_UNICODETEXT` bytes, its NUL included.
    fn utf16(text: &str) -> Vec<u8> {
        text.encode_utf16()
            .chain([0])
            .flat_map(u16::to_le_bytes)
            .collect()
    }

    /// The bytes of `units`, as the fake stores a written text.
    fn bytes_of(units: &[u16]) -> Vec<u8> {
        units.iter().flat_map(|unit| unit.to_le_bytes()).collect()
    }

    fn spec(text: &str, keys: &[VIRTUAL_KEY], restore_delay_ms: u64) -> PasteSpec {
        PasteSpec::new(text, keys.to_vec(), restore_delay_ms).expect("a valid spec")
    }

    /// The clipboard of most tests: "ORIG" as text and an HTML copy, put
    /// there by `OWNER`.
    fn original() -> FakeClipboard {
        FakeClipboard::new()
            .with_name(HTML, "HTML Format")
            .with_content(
                OWNER,
                &[(CF_UNICODETEXT, &utf16("ORIG")), (HTML, b"<b>ORIG</b>")],
            )
    }

    /// The four markers a phrase carries, as the fake registered them.
    fn phrase_markers(clipboard: &FakeClipboard) -> Vec<u32> {
        Marker::ALL[..4]
            .iter()
            .map(|&marker| {
                clipboard
                    .marker_id(marker)
                    .expect("the marker is registered")
            })
            .collect()
    }

    /// What the clipboard holds once the paste of `text` set it.
    fn phrase(clipboard: &FakeClipboard, text: &str) -> Vec<(u32, Vec<u8>)> {
        let mut content = vec![(
            CF_UNICODETEXT,
            bytes_of(&crate::data::paste::clipboard_text(text)),
        )];
        content.extend(
            phrase_markers(clipboard)
                .into_iter()
                .map(|id| (id, vec![0; 4])),
        );
        content
    }

    /// What the clipboard holds once `saved` was put back: the formats in
    /// their order, the four markers and the daemon's own.
    fn restored(clipboard: &FakeClipboard, saved: &[(u32, &[u8])]) -> Vec<(u32, Vec<u8>)> {
        let mut content: Vec<(u32, Vec<u8>)> = saved
            .iter()
            .map(|(format, bytes)| (*format, bytes.to_vec()))
            .collect();
        content.extend(
            Marker::ALL
                .iter()
                .map(|&marker| (clipboard.marker_id(marker).expect("registered"), vec![0; 4])),
        );
        content
    }

    /// The restored copy of [`original`].
    fn original_restored(clipboard: &FakeClipboard) -> Vec<(u32, Vec<u8>)> {
        restored(
            clipboard,
            &[
                (CF_UNICODETEXT, &utf16("ORIG")),
                (HTML, b"<b>ORIG</b>"),
                (CF_LOCALE, &LOCALE),
            ],
        )
    }

    fn logged(lines: &[String], text: &str) -> bool {
        lines.iter().any(|line| line.contains(text))
    }

    /// A Paste as its run keeps it between the session's tries.
    struct Job {
        spec: PasteSpec,
        attempt: Option<Attempt>,
        /// The deadline of the wait it is in, if any.
        wait: Option<Instant>,
        /// What each try returned, in order.
        steps: Vec<Step>,
        done: bool,
    }

    impl Job {
        fn new(spec: PasteSpec) -> Self {
            Self {
                spec,
                attempt: None,
                wait: None,
                steps: Vec::new(),
                done: false,
            }
        }

        /// A Paste of `text` with `Ctrl+V` and the default delay.
        fn text(text: &str) -> Self {
            Self::new(spec(text, &[VK_CONTROL, VK_V], 500))
        }
    }

    /// One paste session on a fake clipboard, a fake desk and a fake clock.
    struct Harness {
        session: PasteSession,
        clipboard: FakeClipboard,
        desk: FakeDesk,
        base: Instant,
        clock: FakeClock,
        /// The answers of `live`, in turn; the last one repeats; `true`
        /// once there are none.
        live: RefCell<VecDeque<bool>>,
    }

    impl Harness {
        fn new(clipboard: FakeClipboard, desk: FakeDesk) -> Self {
            Self::with_clock(clipboard, desk, FakeClock::default())
        }

        fn with_clock(clipboard: FakeClipboard, desk: FakeDesk, clock: FakeClock) -> Self {
            Self {
                session: PasteSession::new(),
                clipboard,
                desk,
                base: Instant::now(),
                clock,
                live: RefCell::new(VecDeque::new()),
            }
        }

        fn with_live(self, answers: &[bool]) -> Self {
            *self.live.borrow_mut() = answers.iter().copied().collect();
            self
        }

        fn offset(&self) -> Duration {
            *self.clock.lock().unwrap_or_else(PoisonError::into_inner)
        }

        fn now(&self) -> Instant {
            self.base + self.offset()
        }

        fn ms(&self, ms: u64) -> Instant {
            self.base + Duration::from_millis(ms)
        }

        /// The clock's time since the start, in milliseconds.
        fn elapsed_ms(&self) -> u128 {
            self.offset().as_millis()
        }

        /// Sets the clock to the start plus `ms`.
        fn at(&self, ms: u64) {
            *self.clock.lock().unwrap_or_else(PoisonError::into_inner) = Duration::from_millis(ms);
        }

        /// One poll as the runs make it: the restore decided, each due job
        /// tried in firing order, the restore made.
        fn pass(&mut self, jobs: &mut [&mut Job]) {
            let Harness {
                session,
                clipboard,
                desk,
                base,
                clock,
                live,
            } = self;
            let ports = Ports {
                clipboard: &*clipboard,
                desk: &*desk,
            };
            let now = || *base + *clock.lock().unwrap_or_else(PoisonError::into_inner);
            let live = || {
                let mut answers = live.borrow_mut();
                if answers.len() > 1 {
                    answers.pop_front().unwrap_or(true)
                } else {
                    answers.front().copied().unwrap_or(true)
                }
            };
            session.tick(&ports, now());
            for job in jobs.iter_mut() {
                if job.done || job.wait.is_some_and(|at| now() < at) {
                    continue;
                }
                let step = session.attempt(&mut job.attempt, &job.spec, &ports, &now, &live);
                job.steps.push(step);
                match step {
                    Step::Done => {
                        job.done = true;
                        job.wait = None;
                    }
                    Step::WaitUntil(at) => job.wait = Some(at),
                }
            }
            session.restore_if_due(&ports, now());
        }

        /// Passes at each next deadline, as the executor's timer wakes it,
        /// until every job is done and no restore waits, or the next
        /// deadline is past `until_ms`.
        fn run(&mut self, jobs: &mut [&mut Job], until_ms: u64) {
            for _ in 0..100_000 {
                self.pass(jobs);
                let next = jobs
                    .iter()
                    .filter(|job| !job.done)
                    .filter_map(|job| job.wait)
                    .chain(self.session.next())
                    .min();
                let Some(next) = next else {
                    return;
                };
                if next > self.ms(until_ms) {
                    return;
                }
                if next > self.now() {
                    *self.clock.lock().unwrap_or_else(PoisonError::into_inner) = next - self.base;
                }
            }
            panic!("the session did not settle");
        }

        /// The clipboard calls so far, the registrations left out.
        fn calls(&self) -> Vec<Call> {
            self.clipboard
                .calls()
                .into_iter()
                .filter(|call| !matches!(call, Call::Register(_)))
                .collect()
        }

        fn count(&self, wanted: impl Fn(&Call) -> bool) -> usize {
            self.calls().iter().filter(|call| wanted(call)).count()
        }
    }

    #[test]
    fn a_paste_saves_empties_sets_the_text_and_markers_then_sends_the_keys() {
        let clipboard = original();
        let seen = Arc::new(Mutex::new(None));
        let desk = {
            let clipboard = clipboard.clone();
            let seen = Arc::clone(&seen);
            FakeDesk::new().with_probe_hook(move || {
                *seen.lock().unwrap_or_else(PoisonError::into_inner) = Some(clipboard.calls());
            })
        };
        let mut h = Harness::new(clipboard.clone(), desk.clone());
        let mut job = Job::text("Kind regards,");

        h.pass(&mut [&mut job]);

        assert_eq!(job.steps, [Step::Done]);
        let m = phrase_markers(&clipboard);
        assert_eq!(
            h.calls(),
            [
                Call::Owner,
                Call::Open(W1),
                Call::Formats,
                Call::Name(HTML),
                Call::Read(CF_UNICODETEXT, SAVE_CAP),
                Call::Read(HTML, SAVE_CAP - 10),
                Call::Read(CF_LOCALE, SAVE_CAP - 21),
                Call::Empty,
                Call::Write(CF_UNICODETEXT),
                Call::Write(m[0]),
                Call::Write(m[1]),
                Call::Write(m[2]),
                Call::Write(m[3]),
                Call::Close,
                Call::SequenceNumber,
            ]
        );
        assert_eq!(
            desk.calls(),
            [
                DeskCall::Foreground,
                DeskCall::ModifierDown,
                DeskCall::Foreground,
                DeskCall::Probe(OWNER, OWNER_PROBE),
                DeskCall::Foreground,
                DeskCall::ModifierDown,
                DeskCall::Press(vec![VK_CONTROL, VK_V]),
            ]
        );
        let at_probe: Vec<Call> = seen
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take()
            .expect("the owner was probed")
            .into_iter()
            .filter(|call| !matches!(call, Call::Register(_)))
            .collect();
        assert_eq!(
            at_probe,
            [Call::Owner],
            "the owner is probed before the open"
        );
        assert_eq!(clipboard.current_owner(), Some(W1));
        assert!(!clipboard.is_open());
    }

    #[test]
    fn the_text_goes_on_the_clipboard_with_crlf_and_one_nul() {
        let clipboard = original();
        let mut h = Harness::new(clipboard.clone(), FakeDesk::new());
        let mut job = Job::text("Kind regards,\nIvan");

        h.pass(&mut [&mut job]);

        let content = clipboard.content();
        let mut text = "Kind regards,\r\nIvan".encode_utf16().collect::<Vec<u16>>();
        text.push(0);
        assert_eq!(content.first(), Some(&(CF_UNICODETEXT, bytes_of(&text))));
        assert_eq!(content, phrase(&clipboard, "Kind regards,\nIvan"));
    }

    #[test]
    fn the_restore_waits_for_the_restore_delay_after_the_keystroke() {
        let clipboard = original();
        let mut h = Harness::new(clipboard.clone(), FakeDesk::new());
        let mut job = Job::text("x");
        h.pass(&mut [&mut job]);
        assert_eq!(h.session.next(), Some(h.ms(500)));

        h.at(499);
        h.pass(&mut []);
        assert_eq!(
            clipboard.content(),
            phrase(&clipboard, "x"),
            "not before 500 ms"
        );

        h.at(500);
        h.pass(&mut []);
        assert_eq!(clipboard.content(), original_restored(&clipboard));
        assert!(!h.session.is_pending());
        assert_eq!(h.session.next(), None);
    }

    #[test]
    fn the_restore_puts_back_every_saved_format_in_enumeration_order_with_the_markers() {
        // Cut critique M3: a program pastes the first format it knows, so
        // the text, read first, goes back where it was, after the richer
        // formats.
        let clipboard = FakeClipboard::new()
            .with_name(HTML, "HTML Format")
            .with_name(RTF, "Rich Text Format")
            .with_content(
                OWNER,
                &[
                    (HTML, b"<b>ORIG</b>"),
                    (RTF, b"{\\rtf1 ORIG}"),
                    (CF_UNICODETEXT, &utf16("ORIG")),
                ],
            );
        let mut h = Harness::new(clipboard.clone(), FakeDesk::new());
        let mut job = Job::text("x");

        h.run(&mut [&mut job], 10_000);

        let reads: Vec<u32> = h
            .calls()
            .into_iter()
            .filter_map(|call| match call {
                Call::Read(format, _) => Some(format),
                _ => None,
            })
            .collect();
        assert_eq!(
            reads,
            [CF_UNICODETEXT, HTML, RTF, CF_LOCALE],
            "the text is read first"
        );
        assert_eq!(
            clipboard.content(),
            restored(
                &clipboard,
                &[
                    (HTML, b"<b>ORIG</b>"),
                    (RTF, b"{\\rtf1 ORIG}"),
                    (CF_UNICODETEXT, &utf16("ORIG")),
                    (CF_LOCALE, &LOCALE),
                ]
            )
        );
    }

    #[test]
    fn the_owner_changes_between_the_deadline_and_the_open_so_nothing_is_emptied() {
        // Research critique I1: the check that the daemon still owns the
        // clipboard is made after its own open succeeded, so a copy that
        // lands while the restore waits for the open is kept.
        let clipboard = original();
        let mut h = Harness::new(clipboard.clone(), FakeDesk::new());
        let mut job = Job::text("x");
        h.pass(&mut [&mut job]);

        let ((), lines) = capture(|| {
            h.at(500);
            clipboard.busy_opens(1);
            h.pass(&mut []);
            clipboard.foreign_copy(OTHER, &[(CF_UNICODETEXT, &utf16("NEW"))]);
            h.at(525);
            h.pass(&mut []);
        });

        assert_eq!(
            h.count(|call| *call == Call::Empty),
            1,
            "only the paste's own"
        );
        assert_eq!(h.count(|call| matches!(call, Call::Open(_))), 3);
        assert_eq!(clipboard.content(), [(CF_UNICODETEXT, utf16("NEW"))]);
        assert!(!h.session.is_pending());
        assert!(
            logged(
                &lines,
                "paste: the clipboard changed after the paste; the previous content was not put back"
            ),
            "{lines:?}"
        );
    }

    #[test]
    fn a_foreign_format_added_without_emptying_stops_the_restore() {
        // Research critique M2.
        let clipboard = original();
        let mut h = Harness::new(clipboard.clone(), FakeDesk::new());
        let mut job = Job::text("x");
        h.pass(&mut [&mut job]);
        clipboard.foreign_add(0xC300, b"added");

        let ((), lines) = capture(|| h.run(&mut [], 10_000));

        assert_eq!(h.count(|call| *call == Call::Empty), 1);
        let mut expected = phrase(&clipboard, "x");
        expected.push((0xC300, b"added".to_vec()));
        assert_eq!(clipboard.content(), expected);
        assert!(
            logged(&lines, "the previous content was not put back"),
            "{lines:?}"
        );
    }

    #[test]
    fn a_copy_during_the_restore_delay_is_kept() {
        let clipboard = original();
        let mut h = Harness::new(clipboard.clone(), FakeDesk::new());
        let mut job = Job::text("x");
        h.pass(&mut [&mut job]);
        h.at(100);
        clipboard.foreign_copy(OTHER, &[(CF_UNICODETEXT, &utf16("NEW"))]);

        let ((), lines) = capture(|| h.run(&mut [], 10_000));

        assert_eq!(h.count(|call| *call == Call::Empty), 1);
        assert_eq!(clipboard.content(), [(CF_UNICODETEXT, utf16("NEW"))]);
        assert_eq!(clipboard.current_owner(), Some(OTHER));
        assert!(
            logged(&lines, "paste: the clipboard changed after the paste"),
            "{lines:?}"
        );
    }

    #[test]
    fn a_private_original_is_not_saved_and_the_clipboard_is_emptied_at_the_restore() {
        // Owner answer: content a password manager marked private is not
        // put back; the clipboard is emptied instead.
        let marker = 0xC200;
        let variants: [(&str, &[u8], bool); 4] = [
            ("ExcludeClipboardContentFromMonitorProcessing", ZEROS, true),
            ("CanIncludeInClipboardHistory", ZEROS, true),
            ("CanIncludeInClipboardHistory", &[0, 0], true),
            ("CanIncludeInClipboardHistory", &[1, 0, 0, 0], false),
        ];
        for (name, data, private) in variants {
            let clipboard = FakeClipboard::new()
                .with_name(HTML, "HTML Format")
                .with_name(marker, name)
                .with_content(
                    OWNER,
                    &[
                        (CF_UNICODETEXT, &utf16("secret")),
                        (HTML, b"<b>secret</b>"),
                        (marker, data),
                    ],
                );
            let mut h = Harness::new(clipboard.clone(), FakeDesk::new());
            let mut job = Job::text("x");

            let ((), lines) = capture(|| h.run(&mut [&mut job], 10_000));

            let reads: Vec<u32> = h
                .calls()
                .into_iter()
                .filter_map(|call| match call {
                    Call::Read(format, _) => Some(format),
                    _ => None,
                })
                .collect();
            if private {
                assert!(
                    reads.iter().all(|&format| format == marker),
                    "{name} {data:?}: {reads:?}"
                );
                assert_eq!(
                    clipboard.content(),
                    [],
                    "{name} {data:?}: emptied, not put back"
                );
                assert!(
                    logged(
                        &lines,
                        "paste: the clipboard held private content; it will be cleared, not put back"
                    ),
                    "{name} {data:?}: {lines:?}"
                );
            } else {
                assert!(
                    reads.contains(&CF_UNICODETEXT),
                    "{name} {data:?}: {reads:?}"
                );
                assert_eq!(
                    clipboard.content(),
                    restored(
                        &clipboard,
                        &[
                            (CF_UNICODETEXT, &utf16("secret")),
                            (HTML, b"<b>secret</b>"),
                            (CF_LOCALE, &LOCALE),
                        ]
                    ),
                    "a history that is allowed is no private original"
                );
            }
            assert_eq!(h.desk.pressed().len(), 1);
        }
    }

    #[test]
    fn our_own_restored_copy_is_not_taken_for_a_private_original() {
        let names = [
            (0xC200, "ExcludeClipboardContentFromMonitorProcessing"),
            (0xC201, "CanIncludeInClipboardHistory"),
            (0xC202, "CanUploadToCloudClipboard"),
            (0xC203, "Clipboard Viewer Ignore"),
            (0xC204, "KeySwitchRsRestored"),
        ];
        let mut clipboard = FakeClipboard::new().with_name(HTML, "HTML Format");
        for (id, name) in names {
            clipboard = clipboard.with_name(id, name);
        }
        let mut content: Vec<(u32, &[u8])> =
            vec![(CF_UNICODETEXT, &[0x4F, 0, 0, 0]), (HTML, b"<b>O</b>")];
        content.extend(names.iter().map(|(id, _)| (*id, ZEROS)));
        let clipboard = clipboard.with_content(W1, &content);
        let mut h = Harness::new(clipboard.clone(), FakeDesk::new());
        let mut job = Job::text("x");

        h.run(&mut [&mut job], 10_000);

        assert_eq!(
            clipboard.content(),
            restored(
                &clipboard,
                &[
                    (CF_UNICODETEXT, &[0x4F, 0, 0, 0]),
                    (HTML, b"<b>O</b>"),
                    (CF_LOCALE, &LOCALE),
                ]
            )
        );
        assert_eq!(
            clipboard.marker_id(Marker::Restored),
            Some(0xC204),
            "the registered id"
        );
    }

    #[test]
    fn an_empty_original_is_emptied_at_the_restore() {
        let clipboard = FakeClipboard::new();
        let mut h = Harness::new(clipboard.clone(), FakeDesk::new());
        let mut job = Job::text("x");

        h.pass(&mut [&mut job]);
        assert_eq!(clipboard.content(), phrase(&clipboard, "x"));
        h.run(&mut [], 10_000);

        assert_eq!(clipboard.content(), []);
        assert_eq!(
            h.count(|call| matches!(call, Call::Read(..) | Call::Name(_))),
            0
        );
        assert_eq!(h.count(|call| *call == Call::Empty), 2);
    }

    #[test]
    fn a_hung_owner_skips_the_paste_and_leaves_the_clipboard_untouched() {
        let clipboard = original();
        let desk = FakeDesk::new().with_answers(&[Probe::NoAnswer]);
        let mut h = Harness::new(clipboard.clone(), desk.clone());
        let mut job = Job::text("x");

        let ((), lines) = capture(|| h.pass(&mut [&mut job]));

        assert_eq!(job.steps, [Step::Done]);
        assert_eq!(h.calls(), [Call::Owner]);
        assert_eq!(desk.probes(), [(OWNER, OWNER_PROBE)]);
        assert_eq!(desk.pressed(), Vec::<Vec<VIRTUAL_KEY>>::new());
        assert_eq!(clipboard.current_owner(), Some(OWNER));
        assert!(
            logged(
                &lines,
                "paste: the program that owns the clipboard did not answer within 100 ms; the paste was skipped"
            ),
            "{lines:?}"
        );
    }

    #[test]
    fn the_owner_probe_skips_our_own_windows() {
        let clipboard = original();
        let desk = FakeDesk::new();
        let mut h = Harness::new(clipboard.clone(), desk.clone());
        let mut first = Job::text("a");
        h.run(&mut [&mut first], 10_000);
        assert_eq!(
            clipboard.current_owner(),
            Some(W1),
            "the restore's window owns it"
        );
        assert_eq!(clipboard.windows().destroyed, [W1]);
        let probes = desk.probes().len();

        let mut second = Job::text("b");
        h.pass(&mut [&mut second]);

        assert_eq!(
            desk.probes().len(),
            probes,
            "no probe of our own last window"
        );
        assert_eq!(desk.pressed().len(), 2);
    }

    #[test]
    fn a_busy_clipboard_is_retried_at_5_10_20_40_80_ms_then_the_paste_is_skipped() {
        let clipboard = original().with_busy_opens(6);
        let mut h = Harness::new(clipboard.clone(), FakeDesk::new());
        let mut job = Job::text("x");

        let ((), lines) = capture(|| h.run(&mut [&mut job], 10_000));

        assert_eq!(
            job.steps,
            [
                Step::WaitUntil(h.ms(5)),
                Step::WaitUntil(h.ms(15)),
                Step::WaitUntil(h.ms(35)),
                Step::WaitUntil(h.ms(75)),
                Step::WaitUntil(h.ms(155)),
                Step::Done,
            ]
        );
        assert_eq!(h.count(|call| matches!(call, Call::Open(_))), 6);
        assert_eq!(h.count(|call| *call == Call::Empty), 0);
        assert_eq!(h.desk.pressed().len(), 0);
        assert!(
            logged(
                &lines,
                "paste: another program kept the clipboard open for 155 ms; the paste was skipped"
            ),
            "{lines:?}"
        );
    }

    #[test]
    fn a_failed_empty_skips_the_paste() {
        let clipboard = original().with_failure(Fail::Empty);
        let mut h = Harness::new(clipboard.clone(), FakeDesk::new());
        let mut job = Job::text("x");

        let ((), lines) = capture(|| h.run(&mut [&mut job], 10_000));

        assert_eq!(h.count(|call| *call == Call::Empty), 1);
        assert_eq!(h.desk.pressed().len(), 0);
        assert_eq!(clipboard.current_owner(), Some(OWNER));
        assert!(!h.session.is_pending());
        assert!(
            logged(&lines, "paste: could not empty the clipboard ("),
            "{lines:?}"
        );
        assert!(logged(&lines, "); the paste was skipped"), "{lines:?}");
    }

    #[test]
    fn a_failed_text_write_restores_at_once_inside_the_same_open_and_sends_no_keys() {
        let clipboard = original().with_failure(Fail::WriteText);
        let mut h = Harness::new(clipboard.clone(), FakeDesk::new());
        let mut job = Job::text("x");

        let ((), lines) = capture(|| h.pass(&mut [&mut job]));

        assert_eq!(h.count(|call| matches!(call, Call::Open(_))), 1, "one open");
        assert_eq!(h.desk.pressed().len(), 0);
        assert!(!h.session.is_pending());
        // The text of the original cannot go back either: the fake fails
        // every text write. Everything else does.
        let mut expected = original_restored(&clipboard);
        expected.retain(|(format, _)| *format != CF_UNICODETEXT);
        assert_eq!(clipboard.content(), expected);
        assert!(
            logged(&lines, "paste: could not put the text on the clipboard ("),
            "{lines:?}"
        );
        assert!(
            logged(
                &lines,
                "); the clipboard was put back and the paste skipped"
            ),
            "{lines:?}"
        );
    }

    #[test]
    fn a_failed_marker_write_only_warns() {
        let clipboard = original().with_failure(Fail::WriteMarker);
        let mut h = Harness::new(clipboard.clone(), FakeDesk::new());
        let mut job = Job::text("x");

        let ((), lines) = capture(|| h.pass(&mut [&mut job]));

        assert_eq!(h.desk.pressed().len(), 1, "the paste goes on");
        let warnings = lines
            .iter()
            .filter(|line| line.contains("paste: could not mark the text as private ("))
            .count();
        assert_eq!(warnings, 1, "{lines:?}");
        assert!(
            logged(&lines, "); it may appear in the clipboard history"),
            "{lines:?}"
        );
    }

    #[test]
    fn a_panic_after_the_empty_still_restores() {
        // Research critique M1: the restore is keyed on "the daemon emptied
        // the clipboard", so a failure after that point still puts the
        // original back.
        let clipboard = original().with_panic_at(Call::Write(CF_UNICODETEXT));
        let mut h = Harness::new(clipboard.clone(), FakeDesk::new());
        let mut job = Job::text("x");

        let unwound = catch_unwind(AssertUnwindSafe(|| h.pass(&mut [&mut job])));

        assert!(unwound.is_err(), "the fake panicked");
        assert!(!clipboard.is_open(), "closed on the unwind");
        assert!(h.session.is_pending(), "the restore waits");
        h.pass(&mut []);
        assert_eq!(clipboard.content(), original_restored(&clipboard));
        assert!(!h.session.is_pending());
    }

    #[test]
    fn the_clipboard_is_closed_on_every_path_panics_included() {
        let plans: [(&str, FakeClipboard, FakeDesk); 9] = [
            ("busy", original().with_busy_opens(100), FakeDesk::new()),
            (
                "empty fails",
                original().with_failure(Fail::Empty),
                FakeDesk::new(),
            ),
            (
                "text write fails",
                original().with_failure(Fail::WriteText),
                FakeDesk::new(),
            ),
            (
                "marker write fails",
                original().with_failure(Fail::WriteMarker),
                FakeDesk::new(),
            ),
            (
                "panic at the enumeration",
                original().with_panic_at(Call::Formats),
                FakeDesk::new(),
            ),
            (
                "panic at the read",
                original().with_panic_at(Call::Read(CF_UNICODETEXT, SAVE_CAP)),
                FakeDesk::new(),
            ),
            (
                "panic at the empty",
                original().with_panic_at(Call::Empty),
                FakeDesk::new(),
            ),
            (
                "panic at the text write",
                original().with_panic_at(Call::Write(CF_UNICODETEXT)),
                FakeDesk::new(),
            ),
            (
                "foreground change after the save",
                original(),
                FakeDesk::new().with_foregrounds(&[0x1234, 0x1234, 0x5678]),
            ),
        ];
        for (plan, clipboard, desk) in plans {
            let mut h = Harness::new(clipboard.clone(), desk);
            let mut job = Job::text("x");
            for _ in 0..200 {
                let _ = catch_unwind(AssertUnwindSafe(|| h.pass(&mut [&mut job])));
                assert!(!clipboard.is_open(), "{plan}: open after a pass");
                let next = job.wait.into_iter().chain(h.session.next()).min();
                let Some(next) = next.filter(|_| !job.done || h.session.is_pending()) else {
                    break;
                };
                if next > h.now() {
                    h.at(u64::try_from((next - h.base).as_millis()).expect("ms"));
                }
            }
            assert!(!clipboard.is_open(), "{plan}");
            assert!(!h.calls().is_empty(), "{plan}: the clipboard was asked");
        }
    }

    #[test]
    fn a_modifier_held_delays_the_keystroke_polled_every_5_ms() {
        let clipboard = original();
        let desk = FakeDesk::new().with_modifiers(&[true, true, true, false]);
        let mut h = Harness::new(clipboard.clone(), desk.clone());
        let mut job = Job::text("x");

        for ms in [0, 5, 10] {
            h.at(ms);
            h.pass(&mut [&mut job]);
            assert_eq!(job.steps.last(), Some(&Step::WaitUntil(h.ms(ms + 5))));
            assert_eq!(
                clipboard.calls(),
                [],
                "no clipboard call while a modifier is held"
            );
        }
        h.at(15);
        h.pass(&mut [&mut job]);

        assert_eq!(job.steps.last(), Some(&Step::Done));
        assert_eq!(desk.pressed(), [vec![VK_CONTROL, VK_V]]);
    }

    #[test]
    fn a_modifier_held_for_3_s_skips_the_paste() {
        let clipboard = original();
        let desk = FakeDesk::new();
        desk.hold_modifiers(true);
        let mut h = Harness::new(clipboard.clone(), desk.clone());
        let mut job = Job::text("x");

        let ((), lines) = capture(|| h.run(&mut [&mut job], 10_000));

        assert!(job.done);
        assert_eq!(h.elapsed_ms(), 3_000);
        assert_eq!(job.steps.len(), 601, "a read every 5 ms");
        assert_eq!(clipboard.calls(), []);
        assert_eq!(desk.pressed().len(), 0);
        assert!(
            logged(
                &lines,
                "paste: a modifier key was still held after 3 s; the paste was skipped"
            ),
            "{lines:?}"
        );
    }

    #[test]
    fn a_foreground_change_before_the_save_skips_the_paste() {
        let clipboard = original();
        let desk = FakeDesk::new().with_foregrounds(&[0x1234, 0x5678]);
        let mut h = Harness::new(clipboard.clone(), desk.clone());
        let mut job = Job::text("x");

        let ((), lines) = capture(|| h.pass(&mut [&mut job]));

        assert!(job.done);
        assert_eq!(h.calls(), []);
        assert_eq!(desk.pressed().len(), 0);
        assert!(
            logged(
                &lines,
                "paste: the foreground window changed before the paste; the paste was skipped"
            ),
            "{lines:?}"
        );
    }

    /// The paste of a harness whose re-check after the save fails: no
    /// keystroke, and the original back in the same pass.
    fn assert_put_back_without_a_keystroke(mut h: Harness) {
        let mut job = Job::text("x");

        let ((), lines) = capture(|| h.pass(&mut [&mut job]));

        assert!(job.done);
        assert_eq!(h.desk.pressed().len(), 0);
        assert_eq!(
            h.count(|call| *call == Call::Empty),
            2,
            "the paste's and the restore's"
        );
        assert_eq!(h.clipboard.content(), original_restored(&h.clipboard));
        assert!(!h.session.is_pending(), "restored in this pass");
        assert!(
            logged(
                &lines,
                "paste: the foreground window, the modifiers or the remapping switch changed after the clipboard was set; it was put back and the paste skipped"
            ),
            "{lines:?}"
        );
    }

    #[test]
    fn a_foreground_change_after_the_save_restores_at_once_and_sends_no_keys() {
        // Research critique I2.
        assert_put_back_without_a_keystroke(Harness::new(
            original(),
            FakeDesk::new().with_foregrounds(&[0x1234, 0x1234, 0x5678]),
        ));
    }

    #[test]
    fn an_epoch_change_after_the_save_restores_at_once_and_sends_no_keys() {
        assert_put_back_without_a_keystroke(
            Harness::new(original(), FakeDesk::new()).with_live(&[true, false]),
        );
    }

    #[test]
    fn a_modifier_pressed_after_the_save_restores_at_once_and_sends_no_keys() {
        assert_put_back_without_a_keystroke(Harness::new(
            original(),
            FakeDesk::new().with_modifiers(&[false, true]),
        ));
    }

    #[test]
    fn no_keystroke_sent_restores_at_once() {
        let desk = FakeDesk::new().with_press_result(ComboSent { sent: 0, of: 4 });
        let clipboard = original();
        let mut h = Harness::new(clipboard.clone(), desk.clone());
        let mut job = Job::text("x");

        let ((), lines) = capture(|| h.pass(&mut [&mut job]));

        assert_eq!(desk.pressed().len(), 1);
        assert_eq!(desk.released(), []);
        assert_eq!(clipboard.content(), original_restored(&clipboard));
        assert!(!h.session.is_pending(), "restored in the same pass");
        assert!(
            logged(
                &lines,
                "paste: the paste keystroke was not sent; the clipboard is put back"
            ),
            "{lines:?}"
        );
    }

    #[test]
    fn a_partial_keystroke_releases_the_missing_keys() {
        // Research critique M1: no key may stay down.
        for (sent, at_once) in [(1, true), (3, false)] {
            let desk = FakeDesk::new().with_press_result(ComboSent { sent, of: 4 });
            let clipboard = original();
            let mut h = Harness::new(clipboard.clone(), desk.clone());
            let mut job = Job::text("x");

            let ((), lines) = capture(|| h.pass(&mut [&mut job]));

            assert_eq!(desk.released(), [(vec![VK_CONTROL, VK_V], sent)], "{sent}");
            if at_once {
                assert!(!h.session.is_pending(), "{sent}: V never went down");
                assert_eq!(clipboard.content(), original_restored(&clipboard));
            } else {
                assert_eq!(h.session.next(), Some(h.ms(500)), "{sent}: V went down");
            }
            assert!(
                logged(
                    &lines,
                    &format!(
                        "paste: the paste keystroke was cut short ({sent} of 4 events); the missing key releases were sent"
                    )
                ),
                "{lines:?}"
            );
        }
    }

    /// The probes of the target window so far.
    fn target_probes(desk: &FakeDesk) -> usize {
        desk.probes()
            .iter()
            .filter(|probe| **probe == (FOREGROUND.0, TARGET_PROBE))
            .count()
    }

    #[test]
    fn a_target_that_does_not_answer_extends_the_restore_by_250_ms_up_to_5_s() {
        // Research critique I5 and the owner's answer: the restore waits
        // while the target window does not answer, at most 5 s.
        let desk = FakeDesk::new().with_answers(&[Probe::NoAnswer; 30]);
        let mut h = Harness::new(FakeClipboard::new(), desk.clone());
        let mut job = Job::text("x");
        h.pass(&mut [&mut job]);

        h.at(500);
        h.pass(&mut []);
        assert_eq!(h.session.next(), Some(h.ms(750)));
        let ((), lines) = capture(|| h.run(&mut [], 10_000));

        assert_eq!(h.elapsed_ms(), 5_000);
        assert_eq!(target_probes(&desk), 18, "at 500, 750, ..., 4750");
        assert!(!h.session.is_pending());
        assert_eq!(h.clipboard.content(), []);
        assert!(
            logged(
                &lines,
                "paste: the target window did not answer for 5 s; the clipboard was put back anyway"
            ),
            "{lines:?}"
        );
    }

    /// The time the restore of a paste at 0 ms happened, with the target's
    /// probe answers `answers`, and the log lines.
    fn restore_time(answers: &[Probe], restore_delay: u64) -> (u128, usize, Vec<String>) {
        let desk = FakeDesk::new().with_answers(answers);
        let mut h = Harness::new(FakeClipboard::new(), desk.clone());
        let mut job = Job::new(spec("x", &[VK_CONTROL, VK_V], restore_delay));
        let ((), lines) = capture(|| h.run(&mut [&mut job], 10_000));
        assert!(!h.session.is_pending());
        assert_eq!(h.clipboard.content(), [], "the restore happened");
        (h.elapsed_ms(), target_probes(&desk), lines)
    }

    #[test]
    fn a_target_that_answers_late_settles_100_ms_before_the_restore() {
        // A thread answers a sent message before it handles the Ctrl+V
        // input queued for it, so a busy target gets 100 ms more.
        let (at, probes, lines) = restore_time(&[Probe::NoAnswer, Probe::Answered], 500);
        assert_eq!((at, probes), (850, 3));
        assert!(!logged(&lines, "did not answer for 5 s"), "{lines:?}");
    }

    #[test]
    fn a_first_probe_answered_late_settles_100_ms_too() {
        // Cut critique M1: an answer that came only after a wait is a busy
        // thread's.
        let (at, probes, _) = restore_time(&[Probe::Late], 500);
        assert_eq!((at, probes), (600, 2));
    }

    #[test]
    fn a_target_that_is_gone_at_the_deadline_restores_at_once() {
        // Cut critique I1: a window that closed (a Win+R box that Enter
        // closed) reads nothing more, so it is no reason to wait.
        let (at, probes, lines) = restore_time(&[Probe::Gone], 500);
        assert_eq!((at, probes), (500, 1));
        assert!(!logged(&lines, "did not answer"), "{lines:?}");
    }

    #[test]
    fn a_restore_delay_of_5_s_is_never_extended() {
        let (at, probes, lines) = restore_time(&[Probe::NoAnswer; 5], 5_000);
        assert_eq!((at, probes), (5_000, 0));
        assert!(!logged(&lines, "did not answer"), "{lines:?}");
    }

    #[test]
    fn a_second_paste_within_the_delay_waits_for_the_restore_then_reuses_the_saved_copy() {
        let clipboard = original();
        let mut h = Harness::new(clipboard.clone(), FakeDesk::new());
        let mut a = Job::text("A");
        h.pass(&mut [&mut a]);
        h.at(100);
        let mut b = Job::text("B");
        h.pass(&mut [&mut b]);
        assert_eq!(b.steps, [Step::WaitUntil(h.ms(500))]);

        h.at(500);
        h.pass(&mut [&mut b]);
        assert!(b.done);
        assert_eq!(clipboard.content(), phrase(&clipboard, "B"));
        assert_eq!(h.session.next(), Some(h.ms(1_000)));
        h.run(&mut [], 10_000);

        assert_eq!(h.elapsed_ms(), 1_000);
        assert_eq!(
            h.count(|call| matches!(call, Call::Read(..))),
            3,
            "only A saved"
        );
        assert_eq!(h.desk.pressed().len(), 2);
        assert_eq!(clipboard.content(), original_restored(&clipboard));
    }

    #[test]
    fn a_second_paste_goes_ahead_after_1_s() {
        let clipboard = original();
        let mut h = Harness::new(clipboard.clone(), FakeDesk::new());
        let mut a = Job::new(spec("A", &[VK_CONTROL, VK_V], 5_000));
        let mut b = Job::text("B");

        let ((), lines) = capture(|| {
            h.pass(&mut [&mut a]);
            h.pass(&mut [&mut b]);
            assert_eq!(b.steps, [Step::WaitUntil(h.ms(1_000))]);
            h.run(&mut [&mut b], 10_000);
        });

        assert_eq!(h.desk.pressed().len(), 2);
        assert_eq!(h.elapsed_ms(), 1_500, "B's own delay after it went at 1 s");
        assert_eq!(clipboard.content(), original_restored(&clipboard));
        assert!(
            logged(
                &lines,
                "paste: a paste went ahead after waiting 1 s for the previous one's restore"
            ),
            "{lines:?}"
        );
    }

    #[test]
    fn a_chained_paste_after_a_copy_saves_afresh() {
        let clipboard = original();
        let mut h = Harness::new(clipboard.clone(), FakeDesk::new());
        let mut a = Job::text("A");
        h.pass(&mut [&mut a]);
        h.at(200);
        clipboard.foreign_copy(OTHER, &[(CF_UNICODETEXT, &utf16("NEW"))]);
        h.at(300);
        let mut b = Job::text("B");

        let ((), lines) = capture(|| h.run(&mut [&mut b], 10_000));

        assert!(b.done);
        assert_eq!(h.desk.pressed().len(), 2);
        assert_eq!(
            clipboard.content(),
            restored(
                &clipboard,
                &[(CF_UNICODETEXT, &utf16("NEW")), (CF_LOCALE, &LOCALE)]
            )
        );
        assert_eq!(
            clipboard.windows(),
            Windows {
                created: vec![W1, W2, W3],
                destroyed: vec![W1, W2]
            }
        );
        assert!(
            logged(&lines, "paste: the clipboard changed after the paste"),
            "{lines:?}"
        );
    }

    #[test]
    fn the_restore_is_retried_every_25_ms_for_1_s_then_given_up() {
        let clipboard = original();
        let mut h = Harness::new(clipboard.clone(), FakeDesk::new());
        let mut job = Job::text("x");
        h.pass(&mut [&mut job]);
        clipboard.busy_opens(1_000);

        let ((), lines) = capture(|| h.run(&mut [], 10_000));

        assert_eq!(h.elapsed_ms(), 1_500);
        assert_eq!(h.count(|call| matches!(call, Call::Open(_))), 1 + 41);
        assert_eq!(
            clipboard.content(),
            phrase(&clipboard, "x"),
            "the phrase stays"
        );
        assert!(!h.session.is_pending());
        assert!(
            logged(
                &lines,
                "paste: another program kept the clipboard open for 1 s; the previous content was not put back"
            ),
            "{lines:?}"
        );
    }

    #[test]
    fn the_window_is_destroyed_after_the_restore_and_the_next_one_created() {
        let clipboard = original();
        let mut h = Harness::new(clipboard.clone(), FakeDesk::new());
        let mut job = Job::text("x");
        h.run(&mut [&mut job], 10_000);
        assert_eq!(
            clipboard.windows(),
            Windows {
                created: vec![W1, W2],
                destroyed: vec![W1]
            }
        );

        // Cut critique M4: a chained paste takes over with the session's
        // window, which owns the clipboard, and the one after the restore
        // is still made once.
        let clipboard = original();
        let mut h = Harness::new(clipboard.clone(), FakeDesk::new());
        let mut a = Job::text("A");
        h.pass(&mut [&mut a]);
        h.at(100);
        let mut b = Job::text("B");
        h.run(&mut [&mut b], 10_000);
        assert_eq!(
            clipboard.windows(),
            Windows {
                created: vec![W1, W2],
                destroyed: vec![W1]
            }
        );
        let opens: Vec<Call> = h
            .calls()
            .into_iter()
            .filter(|call| matches!(call, Call::Open(_)))
            .collect();
        assert_eq!(opens, [Call::Open(W1); 3], "A's, B's and the restore's");
    }

    #[test]
    fn a_window_that_cannot_be_created_skips_the_paste_and_is_tried_again_next_time() {
        let clipboard = original().with_failure(Fail::Create);
        let mut h = Harness::new(clipboard.clone(), FakeDesk::new());
        let mut first = Job::text("a");

        let ((), lines) = capture(|| h.pass(&mut [&mut first]));

        assert!(first.done);
        assert_eq!(h.calls(), []);
        assert_eq!(h.desk.pressed().len(), 0);
        assert!(
            logged(&lines, "paste: could not create the clipboard window ("),
            "{lines:?}"
        );
        assert!(logged(&lines, "); the paste was skipped"), "{lines:?}");

        clipboard.clear_failure(Fail::Create);
        let mut second = Job::text("b");
        h.pass(&mut [&mut second]);
        assert_eq!(clipboard.windows().created, [W1]);
        assert_eq!(h.desk.pressed().len(), 1);
    }

    #[test]
    fn the_save_stops_at_the_50_ms_budget_text_first() {
        // Research critique I2: no other format than the text is read once
        // the save took 50 ms; one read already in flight is Windows'.
        let clock = FakeClock::default();
        let cost = Duration::from_millis(30);
        let clipboard = FakeClipboard::new()
            .with_name(HTML, "HTML Format")
            .with_name(RTF, "Rich Text Format")
            .with_content(
                OWNER,
                &[
                    (RTF, b"{\\rtf1 O}"),
                    (HTML, b"<b>O</b>"),
                    (CF_UNICODETEXT, &utf16("O")),
                ],
            )
            .with_read_cost(CF_UNICODETEXT, cost, clock.clone())
            .with_read_cost(HTML, cost, clock.clone())
            .with_read_cost(RTF, cost, clock.clone());
        let mut h = Harness::with_clock(clipboard.clone(), FakeDesk::new(), clock);
        let mut job = Job::text("x");

        let ((), lines) = capture(|| h.pass(&mut [&mut job]));

        let reads: Vec<u32> = h
            .calls()
            .into_iter()
            .filter_map(|call| match call {
                Call::Read(format, _) => Some(format),
                _ => None,
            })
            .collect();
        assert_eq!(
            reads,
            [CF_UNICODETEXT, RTF],
            "the text, then one read within 50 ms"
        );
        assert!(
            logged(
                &lines,
                "paste: 2 of 4 clipboard formats will not be restored (too large, too slow or unreadable)"
            ),
            "{lines:?}"
        );
        h.run(&mut [], 10_000);
        assert_eq!(
            clipboard.content(),
            restored(
                &clipboard,
                &[(RTF, b"{\\rtf1 O}"), (CF_UNICODETEXT, &utf16("O"))]
            )
        );
    }

    #[test]
    fn the_save_stops_at_64_mib() {
        let clipboard = original().with_declared_size(HTML, 65 << 20);
        let mut h = Harness::new(clipboard.clone(), FakeDesk::new());
        let mut job = Job::text("x");

        let ((), lines) = capture(|| h.run(&mut [&mut job], 10_000));

        let reads: Vec<Call> = h
            .calls()
            .into_iter()
            .filter(|call| matches!(call, Call::Read(..)))
            .collect();
        assert_eq!(
            reads,
            [
                Call::Read(CF_UNICODETEXT, SAVE_CAP),
                Call::Read(HTML, SAVE_CAP - 10),
                Call::Read(CF_LOCALE, SAVE_CAP - 10),
            ]
        );
        assert_eq!(
            clipboard.content(),
            restored(
                &clipboard,
                &[(CF_UNICODETEXT, &utf16("ORIG")), (CF_LOCALE, &LOCALE)]
            )
        );
        assert!(
            logged(
                &lines,
                "paste: 1 of 3 clipboard formats will not be restored"
            ),
            "{lines:?}"
        );
    }

    #[test]
    fn a_save_that_cannot_reserve_memory_loses_that_format_only() {
        // Research critique M8.
        let clipboard = original().with_reserve_failure(HTML);
        let mut h = Harness::new(clipboard.clone(), FakeDesk::new());
        let mut job = Job::text("x");

        let ((), lines) = capture(|| h.run(&mut [&mut job], 10_000));

        assert_eq!(
            clipboard.content(),
            restored(
                &clipboard,
                &[(CF_UNICODETEXT, &utf16("ORIG")), (CF_LOCALE, &LOCALE)]
            )
        );
        assert!(
            logged(
                &lines,
                "paste: 1 of 3 clipboard formats will not be restored"
            ),
            "{lines:?}"
        );
    }

    #[test]
    fn the_marker_ids_are_registered_once_per_daemon() {
        let clipboard = original();
        let mut h = Harness::new(clipboard.clone(), FakeDesk::new());
        let mut first = Job::text("a");
        h.run(&mut [&mut first], 10_000);
        let mut second = Job::text("b");
        h.run(&mut [&mut second], 10_000);

        let registered: Vec<Call> = clipboard
            .calls()
            .into_iter()
            .filter(|call| matches!(call, Call::Register(_)))
            .collect();
        assert_eq!(
            registered,
            Marker::ALL.map(Call::Register),
            "five, for two sessions"
        );
        assert_eq!(h.desk.pressed().len(), 2);
    }

    #[test]
    fn a_paste_without_a_foreground_window_is_skipped() {
        let desk = FakeDesk::new().with_foregrounds(&[0]);
        let mut h = Harness::new(original(), desk.clone());
        let mut job = Job::text("x");

        let ((), lines) = capture(|| h.pass(&mut [&mut job]));

        assert!(job.done);
        assert_eq!(desk.calls(), [DeskCall::Foreground]);
        assert_eq!(h.clipboard.calls(), []);
        assert!(
            logged(&lines, "paste: no foreground window; the paste was skipped"),
            "{lines:?}"
        );
    }

    const SNIPPET: &str = "PASTE-SNIPPET-7Q";
    const ORIGINAL: &str = "ORIG-CONTENT-4K";

    /// A clipboard whose original names `ORIGINAL` in its text and HTML.
    fn secret_original() -> FakeClipboard {
        FakeClipboard::new()
            .with_name(HTML, "HTML Format")
            .with_content(
                OWNER,
                &[
                    (CF_UNICODETEXT, &utf16(ORIGINAL)),
                    (HTML, format!("<b>{ORIGINAL}</b>").as_bytes()),
                ],
            )
    }

    #[test]
    fn no_log_line_contains_the_text_or_the_clipboard_content() {
        // The roadmap's acceptance: no log line contains the phrase or the
        // clipboard's content, nor a format's name or id.
        let mut pressed = 0;
        let ((), lines) = capture(|| {
            // Test 1's script, and its restore.
            let mut h = Harness::new(secret_original(), FakeDesk::new());
            h.run(&mut [&mut Job::text(SNIPPET)], 10_000);
            pressed += h.desk.pressed().len();
            // Test 5's: a copy lands while the restore waits for its open.
            let clipboard = secret_original();
            let mut h = Harness::new(clipboard.clone(), FakeDesk::new());
            h.pass(&mut [&mut Job::text(SNIPPET)]);
            h.at(500);
            clipboard.busy_opens(1);
            h.pass(&mut []);
            clipboard.foreign_copy(OTHER, &[(CF_UNICODETEXT, &utf16(ORIGINAL))]);
            h.run(&mut [], 10_000);
            pressed += h.desk.pressed().len();
            // Test 8's: a private original.
            let clipboard = FakeClipboard::new()
                .with_name(0xC200, "ExcludeClipboardContentFromMonitorProcessing")
                .with_content(
                    OWNER,
                    &[(CF_UNICODETEXT, &utf16(ORIGINAL)), (0xC200, &[0; 4])],
                );
            let mut h = Harness::new(clipboard, FakeDesk::new());
            h.run(&mut [&mut Job::text(SNIPPET)], 10_000);
            pressed += h.desk.pressed().len();
            // Test 15's: the text write fails.
            let mut h = Harness::new(
                secret_original().with_failure(Fail::WriteText),
                FakeDesk::new(),
            );
            h.run(&mut [&mut Job::text(SNIPPET)], 10_000);
            // Test 26's: a keystroke cut short.
            let mut h = Harness::new(
                secret_original(),
                FakeDesk::new().with_press_result(ComboSent { sent: 1, of: 4 }),
            );
            h.run(&mut [&mut Job::text(SNIPPET)], 10_000);
            pressed += h.desk.pressed().len();
        });

        assert_eq!(pressed, 4, "the pastes ran");
        assert!(!lines.is_empty(), "the scripts logged");
        let banned = [
            "SNIPPET",
            "ORIG-CONTENT",
            "HTML Format",
            "0xC0",
            "0xc0",
            "C010",
            "c010",
            "49168",
        ];
        for line in &lines {
            for word in banned {
                assert!(!line.contains(word), "{word:?} in {line}");
            }
        }
    }

    /// Whether `rendered` holds no 3-character run of `secret`.
    fn shows_no_part_of(rendered: &str, secret: &str) -> bool {
        let chars: Vec<char> = secret.chars().collect();
        chars.windows(3).all(|window| {
            let part: String = window.iter().collect();
            !rendered.contains(&part)
        })
    }

    #[test]
    fn a_session_debugs_no_part_of_its_text_or_content() {
        let mut h = Harness::new(secret_original(), FakeDesk::new());
        h.pass(&mut [&mut Job::text(SNIPPET)]);
        assert!(
            h.session.is_pending(),
            "the saved copy waits for its restore"
        );

        for rendered in [format!("{:?}", h.session), format!("{:#?}", h.session)] {
            assert!(shows_no_part_of(&rendered, SNIPPET), "{rendered}");
            assert!(shows_no_part_of(&rendered, ORIGINAL), "{rendered}");
            assert!(!rendered.contains("HTML Format"), "{rendered}");
        }
    }
}

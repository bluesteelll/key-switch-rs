//! A waitable timer for a thread that waits for its own deadlines: the
//! action executor's `Delay` steps today, the gesture timers of roadmap item
//! 2.2 later. It is a kernel object, so a thread waits on it and on its
//! messages at once (`MsgWaitForMultipleObjectsEx`), and no message loop
//! that code on the thread runs can take its signal away.
//!
//! Where Windows offers one (Windows 10 version 1803 and later) it is a
//! high-resolution timer, which signals within about a millisecond of its
//! due time, as `std::thread::sleep` does. A thread timer (`SetTimer`) or a
//! plain waitable timer signals on a tick of the system clock instead,
//! about 15.6 ms apart.

use std::time::Duration;

use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::Threading::{
        CREATE_WAITABLE_TIMER_HIGH_RESOLUTION, CancelWaitableTimer, CreateWaitableTimerExW,
        SetWaitableTimer, TIMER_ALL_ACCESS,
    },
};
use windows::core::PCWSTR;

/// The longest a timer is set for at once, about 24.8 days. A later deadline
/// is reached by setting the timer again when it signals, which keeps the
/// due time far from the end of its 64-bit count of 100 ns.
pub const LONGEST: Duration = Duration::from_millis(0x7FFF_FFFF);

/// A synchronization timer: a wait that its signal satisfies also resets
/// it, so a signal wakes one wait. Not `Send`: it belongs to the thread that
/// made it. The handle is closed when the value is dropped.
pub struct WaitableTimer {
    handle: HANDLE,
    high_resolution: bool,
}

impl WaitableTimer {
    /// A high-resolution timer, or a plain one where Windows has no
    /// high-resolution timer (before Windows 10 version 1803). It is not
    /// set.
    pub fn new() -> windows::core::Result<Self> {
        Self::create(CREATE_WAITABLE_TIMER_HIGH_RESOLUTION)
    }

    /// A timer made with `flags`, or a plain one where Windows refuses them,
    /// as it refuses `CREATE_WAITABLE_TIMER_HIGH_RESOLUTION` before Windows
    /// 10 version 1803.
    fn create(flags: u32) -> windows::core::Result<Self> {
        match open(flags) {
            Ok(handle) => Ok(Self {
                handle,
                high_resolution: flags & CREATE_WAITABLE_TIMER_HIGH_RESOLUTION != 0,
            }),
            Err(_) => open(0).map(|handle| Self {
                handle,
                high_resolution: false,
            }),
        }
    }

    /// A value whose handle is an event, not a timer, so that Windows
    /// refuses to set it: the seam for a gesture timer that cannot be set.
    #[cfg(test)]
    pub(crate) fn not_a_timer() -> Self {
        // SAFETY: no security attributes and a null name, so the call reads
        // no memory of ours; the handle goes straight into the value that
        // closes it.
        let handle = unsafe {
            windows::Win32::System::Threading::CreateEventW(None, false, false, PCWSTR::null())
        }
        .expect("an event is made");
        Self {
            handle,
            high_resolution: false,
        }
    }

    /// Whether this is a high-resolution timer rather than one on the
    /// system clock tick.
    pub fn is_high_resolution(&self) -> bool {
        self.high_resolution
    }

    /// The handle to wait on, open as long as `self` lives.
    pub fn handle(&self) -> HANDLE {
        self.handle
    }

    /// Signals the timer once `after` has passed, rounded up to 100 ns and
    /// cut to [`LONGEST`]. It replaces the due time the timer had.
    pub fn set(&self, after: Duration) -> windows::core::Result<()> {
        let due = due_time(after);
        // SAFETY: `self.handle` is the open timer handle this value owns;
        // `due` is a local that outlives the call, which only reads it; no
        // completion routine is passed, so nothing of ours is called later.
        unsafe { SetWaitableTimer(self.handle, &due, 0, None, None, false) }
    }

    /// Stops the timer. A signal it gave already stays until a wait takes
    /// it.
    pub fn cancel(&self) -> windows::core::Result<()> {
        // SAFETY: `self.handle` is the open timer handle this value owns;
        // the call takes it by value and reads no memory of ours.
        unsafe { CancelWaitableTimer(self.handle) }
    }
}

impl Drop for WaitableTimer {
    fn drop(&mut self) {
        // SAFETY: `self.handle` is open and owned by this value alone, which
        // closes it once, here, and never uses it again.
        let _ = unsafe { CloseHandle(self.handle) };
    }
}

/// A new unnamed timer made with `flags`, which its caller owns.
fn open(flags: u32) -> windows::core::Result<HANDLE> {
    // SAFETY: no security attributes and a null name, so the call reads no
    // memory of ours; the handle it returns goes straight into the
    // WaitableTimer that closes it.
    unsafe { CreateWaitableTimerExW(None, PCWSTR::null(), flags, TIMER_ALL_ACCESS.0) }
}

/// `after` as the due time `SetWaitableTimer` takes: negative, which means
/// relative to now, in units of 100 ns. Rounded up, so that the timer never
/// signals before `after` by its own count; at least one unit, since zero
/// would mean an absolute time; at most [`LONGEST`].
pub(crate) fn due_time(after: Duration) -> i64 {
    let units = after.min(LONGEST).as_nanos().div_ceil(100).max(1);
    -i64::try_from(units).expect("invariant: LONGEST in units of 100 ns fits in an i64")
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use windows::Win32::Foundation::{WAIT_OBJECT_0, WAIT_TIMEOUT};
    use windows::Win32::System::Threading::WaitForSingleObject;

    use super::*;

    /// Waits up to `ms` for `timer` to signal; true if it did.
    fn signals_within(timer: &WaitableTimer, ms: u32) -> bool {
        // SAFETY: the handle is open as long as `timer`, which outlives the
        // call; the call takes it by value and reads no memory of ours.
        let woke = unsafe { WaitForSingleObject(timer.handle(), ms) };
        assert!(
            woke == WAIT_OBJECT_0 || woke == WAIT_TIMEOUT,
            "the wait failed: {woke:?}"
        );
        woke == WAIT_OBJECT_0
    }

    #[test]
    fn due_time_is_relative_rounds_up_to_100_ns_and_stops_at_longest() {
        assert_eq!(due_time(Duration::ZERO), -1, "zero would be absolute");
        assert_eq!(due_time(Duration::from_nanos(1)), -1);
        assert_eq!(due_time(Duration::from_nanos(100)), -1);
        assert_eq!(due_time(Duration::from_nanos(101)), -2);
        assert_eq!(due_time(Duration::from_millis(5)), -50_000);
        let longest = -i64::try_from(LONGEST.as_nanos() / 100).expect("fits");
        assert_eq!(due_time(LONGEST), longest);
        assert_eq!(due_time(Duration::MAX), longest);
        assert!(
            LONGEST < Duration::from_millis(u64::from(u32::MAX)),
            "no single wait of u32::MAX ms or more"
        );
    }

    #[test]
    fn a_timer_signals_once_after_its_due_time() {
        let timer = WaitableTimer::new().expect("a timer is made");
        assert!(
            timer.is_high_resolution(),
            "Windows 10 1803 and later make a high-resolution timer"
        );

        let begun = Instant::now();
        timer
            .set(Duration::from_millis(5))
            .expect("the timer is set");
        assert!(signals_within(&timer, 1_000), "the timer signals");
        let took = begun.elapsed();
        assert!(
            took >= Duration::from_millis(5),
            "it signalled early: {took:?}"
        );
        assert!(
            !signals_within(&timer, 0),
            "the wait took the signal: it wakes one wait"
        );
    }

    #[test]
    fn setting_a_timer_again_replaces_its_due_time() {
        let timer = WaitableTimer::new().expect("a timer is made");
        timer
            .set(Duration::from_secs(60))
            .expect("the timer is set");
        timer
            .set(Duration::from_millis(5))
            .expect("the timer is set again");
        assert!(signals_within(&timer, 1_000), "the new due time counts");
    }

    #[test]
    fn a_cancelled_timer_does_not_signal() {
        let timer = WaitableTimer::new().expect("a timer is made");
        timer
            .set(Duration::from_millis(20))
            .expect("the timer is set");
        timer.cancel().expect("the timer is cancelled");
        assert!(!signals_within(&timer, 100));
    }

    #[test]
    fn a_timer_whose_flags_windows_refuses_is_a_plain_one() {
        // Windows refuses a flag it does not know, as it refuses the
        // high-resolution flag before Windows 10 version 1803.
        let unknown = 0x8000_0000 | CREATE_WAITABLE_TIMER_HIGH_RESOLUTION;
        let timer = WaitableTimer::create(unknown).expect("a plain timer is made instead");
        assert!(!timer.is_high_resolution());

        timer
            .set(Duration::from_millis(1))
            .expect("the timer is set");
        assert!(signals_within(&timer, 1_000), "the plain timer signals");
    }
}

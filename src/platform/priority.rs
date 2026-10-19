//! The priority of the daemon's main thread while it runs its main loop
//! (roadmap item 2.2): the thread that runs the low-level keyboard hook, on
//! which every keystroke on the desktop waits.
//!
//! Microsoft's guidance (Scheduling Priorities) is to run "the process's
//! input thread" at `THREAD_PRIORITY_ABOVE_NORMAL` or
//! `THREAD_PRIORITY_HIGHEST`, and a high-priority thread only briefly: the
//! main thread blocks between events and holds no lock that a lower thread
//! could hold up. `THREAD_PRIORITY_HIGHEST` is base priority 10 in a normal
//! process; `THREAD_PRIORITY_TIME_CRITICAL`, which AutoHotkey's hook thread
//! takes, is above 11, which "interferes with the normal operation of the
//! operating system" (SetThreadPriority). The latency bench's row R8 measured
//! the delivery of a sent message under full CPU load at both priorities
//! before this landed.

use std::marker::PhantomData;

use windows::Win32::System::Threading::{
    GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_HIGHEST, THREAD_PRIORITY_NORMAL,
};

/// The calling thread at `THREAD_PRIORITY_HIGHEST`, until dropped, which
/// sets it back to `THREAD_PRIORITY_NORMAL`. Not `Send`: it belongs to the
/// thread it raised.
pub struct RaisedPriority {
    _thread_bound: PhantomData<*const ()>,
}

impl RaisedPriority {
    /// Raises the calling thread to `THREAD_PRIORITY_HIGHEST`.
    pub fn raise() -> windows::core::Result<Self> {
        // SAFETY: GetCurrentThread returns the calling thread's pseudo
        // handle, which needs no closing; SetThreadPriority takes it and the
        // priority by value and changes only that thread's priority.
        unsafe { SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST) }?;
        Ok(Self {
            _thread_bound: PhantomData,
        })
    }
}

impl Drop for RaisedPriority {
    fn drop(&mut self) {
        // SAFETY: as in `raise`; this value never leaves the thread it
        // raised, so the pseudo handle names that thread.
        let _ = unsafe { SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_NORMAL) };
    }
}

#[cfg(test)]
mod tests {
    use windows::Win32::System::Threading::{GetCurrentThread, GetThreadPriority};

    use crate::platform::priority::RaisedPriority;

    /// The calling thread's priority, relative to its process's class.
    fn priority() -> i32 {
        // SAFETY: the pseudo handle of the calling thread needs no closing,
        // and the call only reads that thread's priority.
        unsafe { GetThreadPriority(GetCurrentThread()) }
    }

    #[test]
    fn raise_sets_highest_and_drop_restores_normal() {
        // On a scratch thread, so that the test thread keeps its priority
        // whatever happens.
        let (inside, after) = std::thread::spawn(|| {
            let raised = RaisedPriority::raise().expect("a thread may raise its own priority");
            let inside = priority();
            drop(raised);
            (inside, priority())
        })
        .join()
        .expect("the scratch thread does not panic");
        assert_eq!(inside, 2, "THREAD_PRIORITY_HIGHEST while raised");
        assert_eq!(after, 0, "THREAD_PRIORITY_NORMAL once dropped");
    }
}

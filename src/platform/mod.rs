//! Wrappers around the Win32 primitives that the daemon's threads use the
//! same way.
//!
//! `timers` is the high-resolution waitable timer that two loops wait on:
//! the action executor's, for its `Delay` steps (roadmap item 2.1), and the
//! main thread's, for its gesture timers (roadmap item 2.2). `priority`
//! raises the main thread while it runs its loop (roadmap item 2.2).
//! `foreground` caches the foreground window's executable for `when:`, and
//! `desktop` tells whether this desktop has keyboard input (roadmap item
//! 2.3).

pub mod desktop;
pub mod foreground;
pub mod priority;
pub mod timers;

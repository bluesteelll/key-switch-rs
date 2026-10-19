//! Wrappers around the Win32 primitives that more than one thread of the
//! daemon waits on the same way.
//!
//! `timers` is the high-resolution waitable timer of the action executor's
//! `Delay` steps, kept here so that the gesture timers of roadmap item 2.2
//! can wait on it too.

pub mod timers;

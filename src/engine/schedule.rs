//! Where the timers the engine asks for go: the [`Scheduler`] trait, and
//! [`TimerQueue`], a plain queue of [`TimerRequest`]s.
//!
//! The daemon's main thread keeps its gesture timers in a `TimerQueue` and
//! wakes for the earliest one with a waitable timer
//! (`hook::main_thread::GestureTimers`). The scenario harness drives a
//! `TimerQueue` on its fake clock (`engine::testing`). Both go through
//! [`Engine::start_timers`](super::Engine::start_timers), which first drops
//! every queued timer whose entry is gone
//! ([`Engine::is_live`](super::Engine::is_live)), so a queue holds the live
//! timers and the ones just started.
//!
//! Nothing here reads a clock: the caller says what time it is.

use std::time::Instant;

use super::{TimerRequest, TimerToken};

/// Keeps the timers the engine asks for and hands each token back to
/// [`Engine::on_timer`](super::Engine::on_timer) once it is due.
pub trait Scheduler {
    /// Starts `request`. False if it could not be started: the engine then
    /// forgets it ([`Engine::forget_timer`](super::Engine::forget_timer)).
    fn start(&mut self, request: TimerRequest) -> bool;

    /// Drops every queued timer whose token `live` rejects.
    fn retain(&mut self, live: &dyn Fn(TimerToken) -> bool);
}

/// The timers started and not yet handed out, in start order.
pub struct TimerQueue {
    pending: Vec<TimerRequest>,
}

impl Default for TimerQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl TimerQueue {
    /// An empty queue with room for the timers of a few presses at once.
    pub fn new() -> Self {
        Self {
            pending: Vec::with_capacity(16),
        }
    }

    pub fn len(&self) -> usize {
        self.pending.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    /// The earliest due time queued.
    pub fn next_due(&self) -> Option<Instant> {
        self.pending.iter().map(|request| request.due).min()
    }

    /// Takes the timer with the earliest due time at or before `now`; of
    /// timers due at the same time, the one started first.
    pub fn take_due(&mut self, now: Instant) -> Option<TimerRequest> {
        let index = self
            .pending
            .iter()
            .enumerate()
            .filter(|(_, request)| request.due <= now)
            .min_by_key(|(_, request)| request.due)
            .map(|(index, _)| index)?;
        // `remove`, not `swap_remove`: the others keep their start order.
        Some(self.pending.remove(index))
    }

    pub fn clear(&mut self) {
        self.pending.clear();
    }
}

impl Scheduler for TimerQueue {
    /// A queue has room for every timer.
    fn start(&mut self, request: TimerRequest) -> bool {
        self.pending.push(request);
        true
    }

    fn retain(&mut self, live: &dyn Fn(TimerToken) -> bool) {
        self.pending.retain(|request| live(request.token));
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::super::{TimerRequest, TimerSlot, TimerToken};
    use crate::engine::{Scheduler, TimerQueue};

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    fn request(seq: u64, due: Instant) -> TimerRequest {
        TimerRequest {
            token: TimerToken {
                generation: 0,
                slot: TimerSlot::Gesture(0x7C),
                seq,
            },
            due,
        }
    }

    /// The `seq` of every timer due by `now`, in the order the queue hands
    /// them out.
    fn due_order(queue: &mut TimerQueue, now: Instant) -> Vec<u64> {
        std::iter::from_fn(|| queue.take_due(now))
            .map(|request| request.token.seq)
            .collect()
    }

    #[test]
    fn the_queue_hands_out_due_timers_earliest_first_and_ties_in_start_order() {
        let t0 = Instant::now();
        let mut queue = TimerQueue::new();
        for (seq, due) in [(1, 30), (2, 10), (3, 30), (4, 20)] {
            assert!(
                queue.start(request(seq, t0 + ms(due))),
                "a queue never refuses"
            );
        }
        assert_eq!(queue.len(), 4);
        assert_eq!(queue.next_due(), Some(t0 + ms(10)));
        assert_eq!(queue.take_due(t0 + ms(9)), None, "nothing is due yet");

        assert_eq!(due_order(&mut queue, t0 + ms(20)), [2, 4]);
        assert_eq!(queue.next_due(), Some(t0 + ms(30)));
        assert_eq!(
            due_order(&mut queue, t0 + ms(30)),
            [1, 3],
            "equal deadlines come out in start order"
        );
        assert!(queue.is_empty());
        assert_eq!(queue.next_due(), None);
    }

    #[test]
    fn retain_keeps_only_live_timers() {
        let t0 = Instant::now();
        let mut queue = TimerQueue::new();
        for seq in 1..=3 {
            queue.start(request(seq, t0 + ms(10 * seq)));
        }

        queue.retain(&|token| token.seq != 2);
        assert_eq!(queue.len(), 2);
        assert_eq!(due_order(&mut queue, t0 + ms(100)), [1, 3]);

        queue.start(request(4, t0));
        queue.clear();
        assert!(queue.is_empty());
        assert_eq!(queue.take_due(t0 + ms(100)), None);
    }
}
